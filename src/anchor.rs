use yew::{prelude::*, html::Scope};
use crate::dragndrop::{DragCallbacks, DragHandler, UnifiedPointerEvent};

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
    drag_callbacks: DragCallbacks,
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
            drag_callbacks: AnchorDragHandler(ctx.link().clone()).into_callbacks(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AnchorMsg::Down(x, y) => {
                self.last_x = x;
                self.last_y = y;
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
                if let Some(callback) = ctx.props().on_end.as_ref() {
                    callback.emit(());
                }
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        return html!{
            <div
                class={ctx.props().class.clone()}
                onmousedown={self.drag_callbacks.mouse.clone()}
                ontouchstart={self.drag_callbacks.touch.clone()}
            />
        };
    }
}

struct AnchorDragHandler(pub Scope<Anchor>);
impl DragHandler for AnchorDragHandler {
    fn on_down(&mut self, event: &UnifiedPointerEvent) {
        self.0.send_message(AnchorMsg::Down(event.client_x(), event.client_y()));
    }

    fn on_move(&mut self, event: &UnifiedPointerEvent) {
        self.0.send_message(AnchorMsg::Move(event.client_x(), event.client_y()));
    }

    fn on_up(&mut self, _event: &UnifiedPointerEvent) {
        self.0.send_message(AnchorMsg::Up);
    }
}