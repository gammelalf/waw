use wasm_bindgen::JsCast;
use web_sys::{MouseEvent, Event, DragEvent};
use gloo::utils::window;
use gloo::events::EventListener;
use yew::{prelude::*, html};

#[derive(Copy, Clone, PartialEq)]
pub struct WindowId(u32);
impl ToString for WindowId {
    fn to_string(&self) -> String {
        format!("waw-window-{}", self.0)
    }
}
impl From<u32> for WindowId {
    fn from(id: u32) -> Self {
        WindowId(id)
    }
}

#[derive(Copy, Clone, Properties, PartialEq)]
pub struct WindowProps {
    pub id: WindowId,
}
pub struct Window {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
pub enum WindowMsg {
    Resize(AnchorPosition, i32, i32)
}
#[derive(Copy, Clone)]
pub enum AnchorPosition {
    Title,
    N, S, W, E,
    NW, NE, SW, SE,
}

impl Component for Window {
    type Message = WindowMsg;
    type Properties = WindowProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Window {
            x: 50,
            y: 50,
            width: 100,
            height: 100,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        use AnchorPosition::*;
        match msg {
            WindowMsg::Resize(Title, dx, dy) => {
                self.x += dx;
                self.y += dy;
                true
            }
            WindowMsg::Resize(pos, dx, dy) => {
                // Change xy to the corner which doesn't move
                if matches!(pos, NW | N | NE) { self.y += self.height; }
                if matches!(pos, NW | W | SW) { self.x += self.width; }

                // Increase or decrease width and height
                if matches!(pos, SW | S | SE) { self.height += dy; }
                if matches!(pos, NE | E | SE) { self.width += dx; }
                if matches!(pos, NW | N | NE) { self.height -= dy; }
                if matches!(pos, NW | W | SW) { self.width -= dx; }

                // Change xy to the top left corner
                if matches!(pos, NW | N | NE) { self.y -= self.height; }
                if matches!(pos, NW | W | SW) { self.x -= self.width; }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        use AnchorPosition::*;
        let on_move = |pos| ctx.link().callback(move |(dx, dy)|
            WindowMsg::Resize(pos, dx, dy)
        );

        return html! {
            <div class="waw-window" style={
                format!("--x: {}px; --y: {}px; --width: {}px; --height: {}px",
                    self.x, self.y, self.width, self.height)
            }
            ondragstart={|event: DragEvent| event.prevent_default()}
            >
                <div id={ctx.props().id.to_string()} class="waw-body">
                    {format!("Hello Window {}", ctx.props().id.to_string())}
                </div>

                <Anchor class={"waw-title"} on_move={on_move(Title)}/>
                <Anchor class={"waw-n"} on_move={on_move(N)}/>
                <Anchor class={"waw-s"} on_move={on_move(S)}/>
                <Anchor class={"waw-w"} on_move={on_move(W)}/>
                <Anchor class={"waw-e"} on_move={on_move(E)}/>
                <Anchor class={"waw-nw"} on_move={on_move(NW)}/>
                <Anchor class={"waw-ne"} on_move={on_move(NE)}/>
                <Anchor class={"waw-sw"} on_move={on_move(SW)}/>
                <Anchor class={"waw-se"} on_move={on_move(SE)}/>
            </div>
        };
    }
}

#[derive(Properties, PartialEq)]
struct AnchorProps {
    class: &'static str,
    on_move: Option<Callback<(i32, i32)>>,
}
struct Anchor {
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
            },
            AnchorMsg::Move(x, y) => {
                let dx = x - self.last_x;
                let dy = y - self.last_y;
                self.last_x = x;
                self.last_y = y;
                if let Some(on_move) = ctx.props().on_move.as_ref() {
                    on_move.emit((dx, dy));
                }

            },
            AnchorMsg::Up => {
                self.on_move = None;
                self.on_up = None;
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        return html!{
            <div class={ctx.props().class} onmousedown={self.on_down.clone()}/>
        };
    }
}
