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

    // Event handler which is assigned to dragenter and dragover
    // to make something a target for dragged windows
    pub make_drop_target: Callback<DragEvent>,
}
pub enum ScreenMsg {
    Resize,
    NewWindow(PendingPromise, WindowInit),
    MoveWindow(usize, DockPosition),
    ToggleWindow(usize),
    CenterWindow(usize),
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

            make_drop_target: Callback::from(|event: DragEvent| {
                if let Some(dt) = event.data_transfer() {
                    if dt.types().includes(&"application/waw".into(), 0) {
                        event.prevent_default();
                    }
                }
            }),
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
                let request_center = init.request_center.unwrap_or(false);
                let mut window: Window = init.into();
                if self.center_dock.is_none() && request_center {
                    self.center_dock = Some(id);
                    window.active = true;
                }
                promise.resolve(window.div.clone());
                self.windows.push(window);
                true
            }
            MoveWindow(id, dock) => {
                if let Some(window) = self.windows.get_mut(id) {
                    if window.center_only() {
                        return false;
                    }
                    window.dock = Some(dock);
                    window.active = true;
                    if self.center_dock == Some(id) {
                        self.center_dock = None;
                    }
                    true
                } else { false }
            }
            ToggleWindow(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    if window.center_only() {
                        if !window.active {
                            self.center_dock = Some(id);
                        }
                    }
                    window.active = !window.active;
                    true
                } else { false }
            }
            CenterWindow(id) => {
                // Remove and close old centered window
                if let Some(old) = self.center_dock {
                    if let Some(old) = self.windows.get_mut(old) {
                        old.active = false;
                    }
                    self.center_dock = None;
                }

                // Set and open new centered window
                if let Some(new) = self.windows.get_mut(id) {
                    new.active = true;
                    self.center_dock = Some(id);
                }

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
            if window.active {Some(self.view_window(ctx, id, window))}
            else {None}
        }).flatten();

        let [top, left, bottom, right] = docks;
        return html!{
            <div class="waw-screen">
                <div class="waw-taskbar">
                    {for self.windows
                        .iter()
                        .enumerate()
                        .map(|(id, window)| self.view_window_icon(ctx, id, window))
                    }
                </div>
                <div class="waw-docks" style={
                    format!("--top: {}px; --left: {}px; --bottom: {}px; --right: {}px;",
                    dock_sizes[0], dock_sizes[1], dock_sizes[2], dock_sizes[3])
                }>
                    <div
                        class="waw-center-dock"
                        ondragenter={self.make_drop_target.clone()}
                        ondragover={self.make_drop_target.clone()}
                        ondrop={ctx.link().batch_callback(|event: DragEvent| {
                            let dt = event.data_transfer()?;
                            let id = dt.get_data("application/waw").ok()?;
                            let id: usize = id.parse().ok()?;
                            event.prevent_default();
                            Some(ScreenMsg::CenterWindow(id))
                        })}
                    >
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
                    {self.view_window(ctx, id, window)}
                </>
            })
            .collect();

        let visible = windows.len() > 0;
        return (visible, html!{
            <div
                class={dock_class}
                ondragenter={self.make_drop_target.clone()}
                ondragover={self.make_drop_target.clone()}
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

    fn view_window_icon(&self, ctx: &Context<Self>, id: usize, window: &Window) -> Html {
        if window.center_only() {
            return html!{
                <img
                    src={window.icon.clone()}
                    alt={window.title.clone()}
                    draggable="false"
                    onclick={ctx.link().callback(move |_: MouseEvent| {
                        ScreenMsg::ToggleWindow(id)
                    })}
                />
            };
        } else {
            return html!{
                <img
                    src={window.icon.clone()}
                    alt={window.title.clone()}
                    draggable="true"
                    ondragstart={Callback::from(move |event: DragEvent| {
                        if let Some(dt) = event.data_transfer() {
                            dt.set_data("application/waw", &id.to_string()).unwrap();
                        }
                    })}
                    onclick={ctx.link().callback(move |_: MouseEvent| {
                        ScreenMsg::ToggleWindow(id)
                    })}
                />
            };
        }
    }

    fn view_window(&self, ctx: &Context<Self>, id: usize, window: &Window) -> Html {
        return html!{
            <div class="waw-window">
                {self.view_window_icon(ctx, id, window)}
                {Html::VRef(window.div.clone().into())}
            </div>
        };
    }

    fn view_dock_selector(&self) -> Html {
        return html!{
            <div class="waw-dock-selector">
                <div>
                    <div/>
                    <div/>
                    <div/>
                    <div/>
                    <div>
                        <div/>
                    </div>
                </div>
            </div>
        };
    }
}