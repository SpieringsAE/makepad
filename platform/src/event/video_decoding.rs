use makepad_shader_compiler::makepad_live_tokenizer::LiveId;

#[derive(Clone, Debug)]
pub struct VideoStreamEvent {
    pub video_id: LiveId,
    pub frame_group: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct VideoDecodingInitializedEvent {
    pub video_id: LiveId,
    pub frame_rate: usize,
    pub video_width: u32,
    pub video_height: u32,
    pub color_format: VideoColorFormat,
    pub duration: u128,
}

#[derive(Clone, Debug)]
pub struct VideoDecodingErrorEvent {
    pub video_id: LiveId,
    pub error: String,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum VideoColorFormat {
    YUV420Planar,
    YUV420SemiPlanar,
    YUV420Flexible,
    #[default]
    Unknown,
}

impl VideoColorFormat {
    pub fn from_str(s: &str) -> Self {
        match s {
            "YUV420Flexible" => VideoColorFormat::YUV420Flexible,
            "YUV420Planar" => VideoColorFormat::YUV420Planar,
            "YUV420SemiPlanar" => VideoColorFormat::YUV420SemiPlanar,
            _ => VideoColorFormat::Unknown,
        }
    }
}
