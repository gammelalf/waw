use std::cmp::max;
use std::default::Default;
use gloo::events::EventListener;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::promise::PendingPromise;
use crate::anchor::Anchor;
use crate::window::{Window, WindowProps};

#[derive(Properties, PartialEq)]
pub struct ScreenProps {
    pub parent: HtmlElement,
}
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub resize_listener: EventListener, // Listen on window for size changes
    // Listening on direct parent would require ResizeObserver,
    // which is unstable and annoying to use in web-sys

    pub windows: Vec<WindowProps>,
    pub next_window_id: u32,

    pub docks: [Dock; 4],
}
pub enum ScreenMsg {
    Resize,
    NewWindow(PendingPromise),
    DeleteWindow(u32),
    ResizeDock(DockPosition, i32, i32),
}
#[derive(Copy, Clone)]
pub enum DockPosition {
    Top, Left, Bottom, Right
}
#[derive(Default)]
pub struct Dock {
    pub pos: i32,
}

impl Component for Screen {
    type Message = ScreenMsg;
    type Properties = ScreenProps;

    fn create(ctx: &Context<Self>) -> Self {
        let scope = ctx.link().clone();
        let resize_listener = EventListener::new(&gloo::utils::window(), "resize", move |_event| {
            scope.send_message(ScreenMsg::Resize);
        });

        let parent: &HtmlElement = &ctx.props().parent;
        Screen {
            width: parent.offset_width() as u32,
            height: parent.offset_height() as u32,
            resize_listener,

            windows: Vec::new(),
            next_window_id: 0,

            docks: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use ScreenMsg::*;
        match msg {
            Resize => {
                let parent = &ctx.props().parent;
                self.width = parent.offset_width() as u32;
                self.height = parent.offset_height() as u32;
                true
            }
            NewWindow(promise) => {
                if self.next_window_id == u32::MAX {
                    promise.reject("Out of ids");
                    return false;
                }
                let id = self.next_window_id;
                self.next_window_id += 1;

                self.windows.push(WindowProps {id: id.into(), min_height: 0, min_width: 0});
                promise.resolve(id);
                true
            },
            DeleteWindow(id) => {
                let mut index = None;
                for (i, props) in self.windows.iter().enumerate() {
                    if props.id == (id as u32).into() {
                        index = Some(i);
                        break;
                    }
                }

                if let Some(index) = index {
                    self.windows.remove(index);
                    true
                } else {
                    false
                }
            }
            ResizeDock(dock, dx, dy) => {
                use DockPosition::*;
                let d = match dock {
                    Top    =>  dy,
                    Left   =>  dx,
                    Bottom => -dy,
                    Right  => -dx,
                };
                self.docks[dock as usize].pos = max(0, self.docks[dock as usize].pos + d);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_move = |dock| ctx.link().callback(move |(dx, dy)|
            ScreenMsg::ResizeDock(dock, dx, dy)
        );
        let windows = self.windows.iter().map(|props| html!{<Window ..props.clone()/>});
        return html!{
            <>
                <div class="waw-docks" style={
                    use DockPosition::*;
                    let docks = &self.docks;
                    format!("--top: {}px; --left: {}px; --bottom: {}px; --right: {}px;",
                    docks[Top as usize].pos, docks[Left as usize].pos,
                    docks[Bottom as usize].pos, docks[Right as usize].pos)
                }>
                    <div class="waw-taskbar"/>
                    <div class="waw-center-dock"/>
                    <div class="waw-left-dock">
                        <Anchor class="waw-e" on_move={on_move(DockPosition::Left)}/>
                        <div class="waw-flex-vertical"/>
                    </div>
                    <div class="waw-right-dock">
                        <Anchor class="waw-w" on_move={on_move(DockPosition::Right)}/>
                        <div class="waw-flex-vertical"/>
                    </div>
                    <div class="waw-top-dock">
                        <Anchor class="waw-s" on_move={on_move(DockPosition::Top)}/>
                        <div class="waw-flex-horizontal"/>
                    </div>
                    <div class="waw-bottom-dock">
                        <Anchor class="waw-n" on_move={on_move(DockPosition::Bottom)}/>
                        <div class="waw-flex-horizontal"/>
                    </div>
                </div>
                {for windows}
            </>
        };
    }
}
