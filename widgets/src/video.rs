use crate::{
    makepad_derive_widget::*, makepad_draw::*, makepad_platform::event::video_playback::*, 
    widget::*,
};

// Currently only supported on Android

// DSL Usage
// source - determines the source for the video playback, can be either:
//  - Network { url: "https://www.someurl.com/video.mkv" }. On Android it supports: HLS, DASH, RTMP, RTSP, and progressive HTTP downloads
//  - Filesystem { path: "/storage/.../DCIM/Camera/video.mp4" }. On Android it requires read permissions that must be granted at runtime.
//  - Dependency { path: dep("crate://self/resources/video.mp4") }. For in-memory videos loaded through LiveDependencies
// is_looping - determines if the video should be played in a loop. defaults to false.
// hold_to_pause - determines if the video should be paused when the user hold the pause button. defaults to false.
// autoplay - determines if the video should start playback when the widget is created. defaults to false.


// Not yet implemented:
// UI
//  - Playback controls
//  - Progress/seek-to bar

// API
//  - Option to restart playback manually when not looping.
//  - Mute/Unmute
//  - Hotswap video source

live_design! {
    VideoBase = {{Video}} {}
}

#[derive(Live)]
pub struct Video {
    // Drawing
    #[live]
    draw_bg: DrawColor,
    #[walk]
    walk: Walk,
    #[live]
    layout: Layout,
    #[live]
    scale: f64,

    // Texture
    #[live]
    source: VideoDataSource,
    #[rust]
    texture: Option<Texture>,
    #[rust]
    texture_handle: Option<u32>,

    // Playback
    #[live(false)]
    is_looping: bool,
    #[live(false)]
    hold_to_pause: bool,
    #[live(false)]
    autoplay: bool,
    #[rust]
    pause_on_first_frame: bool,
    #[rust]
    playback_state: PlaybackState,

    // Original video metadata
    #[rust]
    video_width: usize,
    #[rust]
    video_height: usize,
    #[rust]
    total_duration: u128,

    #[rust]
    id: LiveId,
}

#[derive(Clone, Default, PartialEq, WidgetRef)]
pub struct VideoRef(WidgetRef);

impl VideoRef {
    pub fn preview_first_frame(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.preview_first_frame(cx);
        }
    }

    pub fn begin_playback(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.begin_playback(cx);
        }
    }

    pub fn pause_playback(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.pause_playback(cx);
        }
    }

    pub fn resume_playback(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.resume_playback(cx);
        }
    }

    // it will finish playback and cleanup decoding resources
    pub fn end_playback(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.end_playback(cx);
        }
    }

    pub fn set_source(&mut self, source: VideoDataSource) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_source(source);
        }
    }
}

#[derive(Clone, Default, WidgetSet)]
pub struct VideoSet(WidgetSet);

impl VideoSet {}

#[derive(Default, PartialEq, Debug)]
enum PlaybackState {
    #[default]
    Unprepared,
    Preparing,
    Prepared,
    Previewing,
    Playing,
    Paused,
    // Completed is only used when not looping, means playback reached end of stream
    Completed,
}

impl LiveHook for Video {
    #[allow(unused)]
    fn before_live_design(cx: &mut Cx) {
        #[cfg(target_os = "android")]
        {
            register_widget!(cx, Video);
        }
    }

    #[allow(unused)]
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        #[cfg(target_os = "android")]
        if self.texture.is_none() {
            let new_texture = Texture::new(cx);
            new_texture.set_format(cx, TextureFormat::VideoRGB);
            self.texture = Some(new_texture);
        }

        let texture = self.texture.as_mut().unwrap();
        self.draw_bg.draw_vars.set_texture(0, &texture);

        self.id = LiveId::unique();
    }
}

#[derive(Clone, WidgetAction)]
pub enum VideoAction {
    None,
}

impl Widget for Video {
    fn redraw(&mut self, cx: &mut Cx) {
        self.draw_bg.redraw(cx);
    }

    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }

    fn draw_walk_widget(&mut self, cx: &mut Cx2d, walk: Walk) -> WidgetDraw {
        self.draw_bg.draw_walk(cx, walk);
        WidgetDraw::done()
    }

    fn handle_widget_event_with(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        dispatch_action: &mut dyn FnMut(&mut Cx, WidgetActionItem),
    ) {
        let uid = self.widget_uid();
        self.handle_event_with(cx, event, &mut |cx, action| {
            dispatch_action(cx, WidgetActionItem::new(action.into(), uid));
        });
    }
}

impl Video {
    pub fn handle_event_with(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        _dispatch_action: &mut dyn FnMut(&mut Cx, VideoAction),
    ) {
        if let Event::VideoPlaybackPrepared(event) = event {
            if event.video_id == self.id {
                self.handle_playback_prepared(cx, event);
            }
        }

        if let Event::VideoTextureUpdated(event) = event {
            if event.video_id == self.id {
                self.redraw(cx);
            }
        }

        if let Event::VideoPlaybackCompleted(event) = event {
            if event.video_id == self.id {
                if !self.is_looping {
                    self.playback_state = PlaybackState::Completed;
                }
            }
        }

        if let Event::TextureHandleReady(event) = event {
            if event.texture_id == self.texture.clone().unwrap().texture_id() {
                self.texture_handle = Some(event.handle);
                if self.autoplay && self.playback_state == PlaybackState::Unprepared {
                    self.prepare_playback(cx);
                }
            }
        }

        self.handle_gestures(cx, event);
        self.handle_activity_events(cx, event);
        self.handle_errors(event);
    }

    fn prepare_playback(&mut self, cx: &mut Cx) {
        if self.texture_handle.is_none() {
            error!("Attempted to prepare playback without an external texture available");
            return;
        }

        if self.playback_state == PlaybackState::Unprepared {
            match &self.source {
                VideoDataSource::Dependency { path }  => {
                    match cx.get_dependency(path.as_str()) {
                        Ok(data) => {
                            cx.prepare_video_playback(
                                self.id,
                                VideoSource::InMemory(data),
                                self.texture_handle.unwrap(),
                                self.autoplay,
                                self.is_looping,
                                self.pause_on_first_frame,
                            );
                            self.playback_state = PlaybackState::Preparing;
                        }
                        Err(e) => {
                            error!(
                                "Attempted to prepare playback: resource not found {} {}",
                                path.as_str(),
                                e
                            );
                        }
                    }
                },
                VideoDataSource::Network { url } => {
                    cx.prepare_video_playback(
                        self.id,
                        VideoSource::Network(url.to_string()),
                        self.texture_handle.unwrap(),
                        self.autoplay,
                        self.is_looping,
                        self.pause_on_first_frame,
                    );
                    self.playback_state = PlaybackState::Preparing;
                },
                VideoDataSource::Filesystem { path } => {
                    cx.prepare_video_playback(
                        self.id,
                        VideoSource::Filesystem(path.to_string()),
                        self.texture_handle.unwrap(),
                        self.autoplay,
                        self.is_looping,
                        self.pause_on_first_frame,
                    );
                    self.playback_state = PlaybackState::Preparing;
                }
            }
        }
    }

    fn handle_playback_prepared(&mut self, cx: &mut Cx, event: &VideoPlaybackPreparedEvent) {
        self.playback_state = PlaybackState::Prepared;
        self.video_width = event.video_width as usize;
        self.video_height = event.video_height as usize;
        self.total_duration = event.duration;

        self.draw_bg
            .set_uniform(cx, id!(video_height), &[self.video_height as f32]);
        self.draw_bg
            .set_uniform(cx, id!(video_width), &[self.video_width as f32]);

        // Debug
        // log!(
        //     "Video id {} - decoding initialized: \n {}x{}px |",
        //     self.id.0,
        //     self.video_width,
        //     self.video_height,
        // );
    }

    fn handle_gestures(&mut self, cx: &mut Cx, event: &Event) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerDown(_fe) => {
                if self.hold_to_pause {
                    self.pause_playback(cx);
                }
            }
            Hit::FingerUp(_fe) => {
                if self.hold_to_pause {
                    self.resume_playback(cx);
                }
            }
            _ => (),
        }
    }

    fn handle_activity_events(&mut self, cx: &mut Cx, event: &Event) {
        match event {
            Event::Pause => self.pause_playback(cx),
            Event::Resume => self.resume_playback(cx),
            _ => (),
        }
    }

    fn handle_errors(&mut self, event: &Event) {
        if let Event::VideoDecodingError(event) = event {
            if event.video_id == self.id {
                error!(
                    "Error decoding video with id {} : {}",
                    self.id.0, event.error
                );
            }
        }
    }

    fn preview_first_frame(&mut self, cx: &mut Cx) {
        if self.playback_state == PlaybackState::Unprepared {
            self.prepare_playback(cx);
            self.pause_on_first_frame = true;
            self.playback_state = PlaybackState::Previewing;
        }
    }

    fn begin_playback(&mut self, cx: &mut Cx) {
        if self.playback_state == PlaybackState::Unprepared {
            self.prepare_playback(cx);
            self.playback_state = PlaybackState::Playing;
        }
    }

    fn pause_playback(&mut self, cx: &mut Cx) {
        if self.playback_state != PlaybackState::Paused {
            cx.pause_video_playback(self.id);
            self.playback_state = PlaybackState::Paused;
        }
    }

    fn resume_playback(&mut self, cx: &mut Cx) {
        if self.playback_state == PlaybackState::Paused {
            cx.resume_video_playback(self.id);
            self.playback_state = PlaybackState::Playing;
        }
    }

    fn end_playback(&mut self, cx: &mut Cx) {
        if self.playback_state != PlaybackState::Unprepared {
            cx.end_video_playback(self.id);
        }
        self.playback_state = PlaybackState::Unprepared;
    }

    fn set_source(&mut self, source: VideoDataSource) {
        if self.playback_state == PlaybackState::Unprepared {
            self.source = source;
        } else {
            error!("Attempted to set source while player state is: {:?}", self.playback_state);
        } 
    }
}

#[derive(Clone, Debug, Live, LiveHook)]
#[live_ignore]
pub enum VideoDataSource {
    #[live {path: LiveDependency::default()}]
    Dependency {path: LiveDependency},
    #[pick {url: "".to_string()}]
    Network {url: String},
    #[live {path: "".to_string()}]
    Filesystem {path: String}
}
