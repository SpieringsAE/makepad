use makepad_widgets::*;

live_design!{
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    
    App = {{App}} {

        ui: <Window>{
            show_bg: true
            width: Fill,
            height: Fill
            
            draw_bg: {
                fn pixel(self) -> vec4 {
                    //return #000
                    return mix(#7, #3, self.pos.y);
                }
            }
            
            body = <View>{
                 
                flow: Down,
                spacing: 20,
                align: {
                    x: 0.5,
                    y: 0.5
                },
                button1 = <Button> {
                    text: "Hello world"
                }
                input1 = <TextInput> {
                    width: 100, height: 30
                    text: "Click to count"
                }
                
                label1 = <Label> {
                    draw_text: {
                        color: #f
                    },
                    text: "Counter: 0"
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] counter: usize,
}

impl LiveHook for App {
    fn before_live_design(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
    }
}

impl App{
    async fn _do_network_request(_cx:CxRef, _ui:WidgetRef, _url:&str)->String{
        "".to_string()
    }
}

impl AppMain for App{
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Draw(event) = event {
            return self.ui.draw_widget_all(&mut Cx2d::new(cx, event));
        }
        let actions = self.ui.handle_widget_event(cx, event);
  
        if self.ui.button(id!(button1)).clicked(&actions) {
            log!("BUTTON CLICKED {}", self.counter); 
            self.counter += 1;
            let label = self.ui.label(id!(label1));
            label.set_text_and_redraw(cx,&format!("Counter: {}", self.counter));
        }
    }
}