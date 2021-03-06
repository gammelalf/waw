use std::cmp::max;
use serde::Deserialize;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use gloo::events::EventListener;
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
    pub dock_windows: [Vec<usize>; 5],
    pub dock_selector: Option<(usize, i32, i32)>,

    // Event handler which is assigned to dragenter and dragover
    // to make something a target for dragged windows
    pub make_drop_target: Callback<DragEvent>,
}
pub enum ScreenMsg {
    Resize,
    NewWindow(PendingPromise, WindowInit),
    MoveWindow(usize, DockPosition),
    OpenSelector(usize, i32, i32),
    CloseSelector(Option<DockPosition>),
    ToggleWindow(usize),
    ResizeDock(DockPosition, i32, i32),
}
#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockPosition {
    Top, Left, Bottom, Right, Center
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
            dock_windows: Default::default(),
            dock_selector: None,

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
                let window: Window = init.into();

                promise.resolve(window.div.clone());

                self.windows.push(window);
                true
            }
            MoveWindow(id, dock) => {
                self.dock_selector = None;
                if let Some(window) = self.windows.get_mut(id) {
                    // Remove moved window from its current dock
                    if let Some(current_dock) = window.current_dock {
                        window.last_dock = current_dock;
                        find_and_delete(&mut self.dock_windows[current_dock as usize], &id);
                    }

                    // Add to new dock and ensure active
                    window.current_dock = Some(dock);
                    self.dock_windows[dock as usize].push(id);
                    true
                } else { false }
            }
            ToggleWindow(id) => {
                if let Some(window) = self.windows.get_mut(id) {
                    // Hide
                    if let Some(current_dock) = window.current_dock {
                        window.current_dock = None;
                        window.last_dock = current_dock;
                        find_and_delete(&mut self.dock_windows[current_dock as usize], &id);
                    }

                    // Show
                    else {
                        window.current_dock = Some(window.last_dock);
                        self.dock_windows[window.last_dock as usize].push(id);
                    }
                    true
                } else { false }
            }
            ResizeDock(dock, dx, dy) => {
                use DockPosition::*;
                let d = match dock {
                    Top    =>  dy,
                    Left   =>  dx,
                    Bottom => -dy,
                    Right  => -dx,
                    Center => return false,
                };
                self.dock_sizes[dock as usize] = max(0, self.dock_sizes[dock as usize] + d);
                true
            }
            OpenSelector(id, x, y) => {
                self.dock_selector = Some((id, x, y));
                true
            }
            CloseSelector(dock) => {
                if let Some((_id, _, _)) = self.dock_selector.take() {
                    if let Some(_dock) = dock {
                        // currently closing the dock with a new dock is handled via MoveWindow
                        panic!("Not Implemented");
                    }
                    true
                } else { false }
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
        let center_dock = self.dock_windows[DockPosition::Center as usize]
            .last()
            .map(|&id| (id, &self.windows[id]))
            .map(|(id, window)| self.view_window(ctx, id, window));

        let [top, left, bottom, right] = docks;
        return html!{
            <div class="waw-screen">
                {self.view_taskbar(ctx)}
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
                            Some(ScreenMsg::MoveWindow(id, DockPosition::Center))
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
    fn view_taskbar(&self, ctx: &Context<Self>) -> Html {
        let windows = self.windows.iter()
            .enumerate()
            .map(|(id, window)| {
                let open = window.current_dock.is_some();
                let menu_open = matches!(self.dock_selector, Some((s_id, _, _)) if s_id == id);
                return html!{
                    <div>
                        if open {
                            <div class="waw-open-indicator"/>
                        } else {
                            <div/>
                        }
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
                        if !menu_open {
                            <div
                                onclick={ctx.link().callback(move |event: MouseEvent| {
                                    let target: HtmlElement = event.target()
                                        .expect("It's a div, see two lines above")
                                        .dyn_into()
                                        .expect("It's a div, see two lines above");
                                    let rect = target.get_bounding_client_rect();
                                    let x = rect.x() + rect.width() / 2.0;
                                    let y = rect.y() + rect.height() / 2.0;
                                    ScreenMsg::OpenSelector(id, x.floor() as i32, y.floor() as i32)
                                })}
                            />
                        } else {
                            <div
                                onclick={ctx.link().callback(move |_: MouseEvent| {
                                    ScreenMsg::CloseSelector(None)
                                })}
                            />
                        }
                        if menu_open {
                            <div>
                                {self.view_dock_selector(ctx).unwrap()}
                            </div>
                        }
                    </div>
                };
            });

        return html!{
            <div class="waw-taskbar">
                {for windows}
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
            Center => unreachable!(),
        };
        let anchor_class = match dock {
            Top    => "waw-s",
            Left   => "waw-e",
            Bottom => "waw-n",
            Right  => "waw-w",
            Center => unreachable!(),
        };

        let windows: Vec<Html> = self.dock_windows[dock as usize]
            .iter()
            .map(|&id| (id, &self.windows[id]))
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

    fn view_window(&self, ctx: &Context<Self>, id: usize, window: &Window) -> Html {
        return html!{
            <div class="waw-window">
                <img
                    class="waw-window-icon"
                    src={window.icon.clone()}
                    alt={window.title.clone()}
                    draggable="false"
                    onclick={ctx.link().callback(move |_: MouseEvent| {
                        ScreenMsg::ToggleWindow(id)
                    })}
                />
                {Html::VRef(window.div.clone().into())}
            </div>
        };
    }

    fn view_dock_selector(&self, ctx: &Context<Self>) -> Option<Html> {
        self.dock_selector.map(|(id, x, y)| {
            let on_click = move |dock| {
                ctx.link().callback(move |_: MouseEvent|
                    ScreenMsg::MoveWindow(id, dock)
                )
            };
            return html!{
                <div class="waw-modal-background" onclick={ctx.link().callback(|_: MouseEvent| {
                    ScreenMsg::CloseSelector(None)
                })}>
                    <div class="waw-dock-selector" style={
                        format!("--x: {}px; --y: {}px", x, y)
                    }>
                        <div onclick={on_click(DockPosition::Top)}/>
                        <div onclick={on_click(DockPosition::Left)}/>
                        <div onclick={on_click(DockPosition::Bottom)}/>
                        <div onclick={on_click(DockPosition::Right)}/>
                        <div onclick={on_click(DockPosition::Center)}/>
                    </div>
                </div>
            };
        })
    }
}

fn find_and_delete<T: PartialEq>(vec: &mut Vec<T>, t : &T) {
    let indexes: Vec<usize> = vec.iter()
        .enumerate()
        .filter(|(_, it)| *it == t)
        .map(|(index, _)| index)
        .collect();
    for index in indexes.into_iter().rev() {
        vec.remove(index);
    }
}