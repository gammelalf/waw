use web_sys::DragEvent;
use yew::prelude::*;
use crate::anchor::Anchor;

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
    #[prop_or(0)]
    pub min_width: i32,
    #[prop_or(0)]
    pub min_height: i32,
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

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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

                // Clamp width and height
                let props = ctx.props();
                if self.width < props.min_width { self.width = props.min_width; }
                if self.height < props.min_height { self.height = props.min_height; }

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
