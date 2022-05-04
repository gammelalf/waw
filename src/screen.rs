use std::cmp::max;
use std::default::Default;
use gloo::events::EventListener;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::promise::PendingPromise;
use crate::anchor::Anchor;
use crate::window::{Window, WindowInit};

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

    pub windows: Vec<Window>,

    pub docks: [Dock; 4],
}
pub enum ScreenMsg {
    Resize,
    NewWindow(PendingPromise, WindowInit),
    MoveWindow(usize, Option<DockPosition>),
    ToggleWindow(usize),
    ResizeDock(DockPosition, i32, i32),
}
#[derive(Copy, Clone, PartialEq, Debug)]
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
        let resize_listener =
        EventListener::new(&gloo::utils::window(), "resize", move |_event| {
            scope.send_message(ScreenMsg::Resize);
        });

        let parent: &HtmlElement = &ctx.props().parent;
        Screen {
            width: parent.offset_width() as u32,
            height: parent.offset_height() as u32,
            resize_listener,

            windows: Vec::new(),

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
            NewWindow(promise, init) => {
                promise.resolve(self.windows.len() as u32);
                self.windows.push(init.into());
                true
            }
            MoveWindow(id, dock) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.dock = dock;
                    window.active = true;
                    true
                } else { false }
            }
            ToggleWindow(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.active = !window.active; true
                } else { false }
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
        return html!{
            <>
                <div class="waw-docks" style={
                    use DockPosition::*;
                    let docks = &self.docks;
                    format!("--top: {}px; --left: {}px; --bottom: {}px; --right: {}px;",
                    docks[Top as usize].pos, docks[Left as usize].pos,
                    docks[Bottom as usize].pos, docks[Right as usize].pos)
                }>
                    {self.view_taskbar(ctx)}
                    <div class="waw-center-dock"/>
                    {self.view_dock(DockPosition::Left, ctx)}
                    {self.view_dock(DockPosition::Right, ctx)}
                    {self.view_dock(DockPosition::Top, ctx)}
                    {self.view_dock(DockPosition::Bottom, ctx)}
                </div>
            </>
        };
    }
}
impl Screen {
    fn view_taskbar(&self, ctx: &Context<Self>) -> Html {
        let icons = self.windows
            .iter()
            .enumerate()
            .map(|(id, window)| html!{
                <img
                    src={window.icon.clone()}
                    alt={window.title.clone()}
                    ondragstart={Callback::from(move |event: DragEvent| {
                        if let Some(dt) = event.data_transfer() {
                            dt.set_data("application/waw", &id.to_string());
                        }
                    })}
                    onclick={ctx.link().callback(move |_: MouseEvent| {
                        ScreenMsg::ToggleWindow(id)
                    })}
                />
            });
        return html!{
            <div class="waw-taskbar">
                {for icons}
            </div>
        };
    }

    fn view_dock(&self, dock: DockPosition, ctx: &Context<Self>) -> Html {
        use DockPosition::*;
        let dock_class = match dock {
            Top    => "waw-top-dock",
            Left   => "waw-left-dock",
            Bottom => "waw-bottom-dock",
            Right  => "waw-right-dock",
        };
        let anchor_class = match dock {
            Top    => "waw-s",
            Left   => "waw-e",
            Bottom => "waw-n",
            Right  => "waw-w",
        };

        let make_drop_target = Callback::from(|event: DragEvent| {
            if let Some(dt) = event.data_transfer() {
                if dt.types().includes(&"application/waw".into(), 0) {
                    event.prevent_default();
                }
            }
        });

        let windows = self.windows
            .iter()
            .enumerate()
            .filter(|(_, window)|
                window.dock == Some(dock)
            )
            .filter(|(_, window)|
                window.active
            )
            .map(|(id, window)| html!{
                <key={id}>
                    {Html::VRef(window.div.clone().into())}
                </>
            });

        return html!{
            <div
                class={dock_class}
                ondragenter={make_drop_target.clone()}
                ondragover={make_drop_target}
                ondrop={ctx.link().batch_callback(move |event: DragEvent| {
                    let dt = event.data_transfer()?;
                    let id = dt.get_data("application/waw").ok()?;
                    let id: usize = id.parse().ok()?;
                    event.prevent_default();
                    Some(ScreenMsg::MoveWindow(id, Some(dock)))
                })}
            >
                <Anchor class={anchor_class}
                    on_move={ctx.link().callback(move |(dx, dy)|
                        ScreenMsg::ResizeDock(dock, dx, dy)
                    )}
                />
                <div class="waw-container">
                    {for windows}
                </div>
            </div>
        };
    }
}