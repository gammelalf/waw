use std::cmp::max;
use gloo::events::EventListener;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::promise::PendingPromise;
use crate::anchor::Anchor;
use crate::drop_zone::DropZone;
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

    pub dock_sizes: [i32; 4],
    pub center_dock: Option<usize>,
}
pub enum ScreenMsg {
    Resize,
    NewWindow(PendingPromise, WindowInit),
    MoveWindow(usize, DockPosition),
    ToggleWindow(usize),
    CenterWindow(Option<usize>),
    ResizeDock(DockPosition, i32, i32),
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DockPosition {
    Top, Left, Bottom, Right
}
impl DockPosition {
    #[inline]
    pub fn array() -> [DockPosition; 4] {
        use DockPosition::*;
        [Top, Left, Bottom, Right]
    }
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
        let width = parent.offset_width() as u32;
        let height = parent.offset_height() as u32;
        Screen {
            width, height, resize_listener,

            windows: Vec::new(),

            dock_sizes: [height as i32 / 10, width as i32 / 10, height as i32 / 5, width as i32 / 5],
            center_dock: None,
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
                let id = self.windows.len();
                if self.center_dock.is_none() && init.request_center == Some(true) {
                    self.center_dock = Some(id);
                }
                self.windows.push(init.into());
                promise.resolve(id as u32);
                true
            }
            MoveWindow(id, dock) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.dock = Some(dock);
                    window.active = true;
                    true
                } else { false }
            }
            ToggleWindow(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    window.active = !window.active; true
                } else { false }
            }
            CenterWindow(id) => {
                self.center_dock = id;
                true
            }
            ResizeDock(dock, dx, dy) => {
                use DockPosition::*;
                let d = match dock {
                    Top    =>  dy,
                    Left   =>  dx,
                    Bottom => -dy,
                    Right  => -dx,
                };
                self.dock_sizes[dock as usize] = max(0, self.dock_sizes[dock as usize] + d);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut dock_sizes = self.dock_sizes;
        let docks: [Html; 4] = DockPosition::array()
            .map(|dock| {
                let (visible, html) = self.view_dock(ctx, dock);
                if !visible {
                    dock_sizes[dock as usize] = 0;
                }
                html
            });
        let center_dock = self.center_dock.map(|id| {
            let window = self.windows.get(id)?;
            return Some(Html::VRef(window.div.clone().into()));
        }).flatten();

        let [top, left, bottom, right] = docks;
        return html!{
            <div class="waw-screen">
                {self.view_taskbar(ctx)}
                <div class="waw-docks" style={
                    format!("--top: {}px; --left: {}px; --bottom: {}px; --right: {}px;",
                    dock_sizes[0], dock_sizes[1], dock_sizes[2], dock_sizes[3])
                }>
                    <div class="waw-center-dock">
                        if let Some(center) = center_dock {
                            {center}
                        }
                    </div>
                    {left}
                    {right}
                    {top}
                    {bottom}
                </div>
            </div>
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

    fn view_dock(&self, ctx: &Context<Self>, dock: DockPosition) -> (bool, Html) {
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

        let windows: Vec<Html> = self.windows
            .iter()
            .enumerate()
            .filter(|(_, window)|
                window.dock == Some(dock)
            )
            .filter(|(_, window)|
                window.active
            )
            .filter(|(id, _)|
                Some(*id) != self.center_dock
            )
            .map(|(id, window)| html!{
                <key={id}>
                    {Html::VRef(window.div.clone().into())}
                </>
            })
            .collect();

        let visible = windows.len() > 0;
        return (visible, html!{
            <div
                class={dock_class}
                ondragenter={make_drop_target.clone()}
                ondragover={make_drop_target}
                ondrop={ctx.link().batch_callback(move |event: DragEvent| {
                    let dt = event.data_transfer()?;
                    let id = dt.get_data("application/waw").ok()?;
                    let id: usize = id.parse().ok()?;
                    event.prevent_default();
                    Some(ScreenMsg::MoveWindow(id, dock))
                })}
            >
                if visible {
                    <Anchor class={anchor_class}
                        on_move={ctx.link().callback(move |(dx, dy)|
                            ScreenMsg::ResizeDock(dock, dx, dy)
                        )}
                    />
                    <div class="waw-container">
                        {for windows.into_iter()}
                    </div>
                } else {
                    <DropZone class="waw-drop-zone" over_class="waw-active"/>
                }
            </div>
        });
    }
}