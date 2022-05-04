use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use gloo::events::EventListener;
use gloo::utils::window;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AnchorProps {
    #[prop_or_default] pub class: Classes,
    #[prop_or_default] pub on_begin: Option<Callback<()>>,
    #[prop_or_default] pub on_move: Option<Callback<(i32, i32)>>,
    #[prop_or_default] pub on_end: Option<Callback<()>>,
}
pub struct Anchor {
    last_x: i32,
    last_y: i32,
    on_down: Callback<MouseEvent>,
    on_move: Option<EventListener>,
    on_up: Option<EventListener>,
}
pub enum AnchorMsg {
    Down(i32, i32),
    Move(i32, i32),
    Up,
}
impl Component for Anchor {
    type Message = AnchorMsg;
    type Properties = AnchorProps;

    fn create(ctx: &Context<Self>) -> Self {
        Anchor {
            last_x: 0,
            last_y: 0,
            on_down: ctx.link().callback(|event: MouseEvent|
                AnchorMsg::Down(event.client_x(), event.client_y())
            ),
            on_move: None,
            on_up: None
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AnchorMsg::Down(x, y) => {
                self.last_x = x;
                self.last_y = y;
                let window = window();

                let scope = ctx.link().clone();
                self.on_move = Some(EventListener::new(&window, "mousemove", move |event: &Event| {
                    let event: &MouseEvent = event.dyn_ref().unwrap();
                    scope.send_message(AnchorMsg::Move(event.client_x(), event.client_y()));
                    if event.buttons() == 0 {
                        scope.send_message(AnchorMsg::Up);
                    }
                }));

                let scope = ctx.link().clone();
                self.on_up = Some(EventListener::new(&window, "mouseup", move |event: &Event| {
                    let event: &MouseEvent = event.dyn_ref().unwrap();
                    scope.send_message(AnchorMsg::Move(event.client_x(), event.client_y()));
                    scope.send_message(AnchorMsg::Up);
                }));

                if let Some(callback) = ctx.props().on_begin.as_ref() {
                    callback.emit(());
                }
            },
            AnchorMsg::Move(x, y) => {
                let dx = x - self.last_x;
                let dy = y - self.last_y;
                self.last_x = x;
                self.last_y = y;
                if let Some(callback) = ctx.props().on_move.as_ref() {
                    callback.emit((dx, dy));
                }

            },
            AnchorMsg::Up => {
                self.on_move = None;
                self.on_up = None;
                if let Some(callback) = ctx.props().on_end.as_ref() {
                    callback.emit(());
                }
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        return html!{
            <div class={ctx.props().class.clone()} onmousedown={self.on_down.clone()}/>
        };
    }
}
