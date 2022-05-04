use gloo::events::EventListener;
use web_sys::HtmlElement;
use yew::{prelude::*, html};

use crate::promise::PendingPromise;
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

    pub top_dock: i32,
    pub left_dock: i32,
    pub bottom_dock: i32,
    pub right_dock: i32,
}
pub enum ScreenMsg {
    Resize,
    NewWindow(PendingPromise),
    DeleteWindow(u32),
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

            top_dock: 50,
            left_dock: 100,
            bottom_dock: 50,
            right_dock: 100,
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
            ScreenMsg::NewWindow(promise) => {
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
            ScreenMsg::DeleteWindow(id) => {
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
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let windows = self.windows.iter().map(|props| html!{<Window ..props.clone()/>});
        return html!{
            <>
                <div class="waw-docks" style={
                    format!("--left: {}px; --top: {}px; --right: {}px; --bottom: {}px;",
                        self.left_dock, self.top_dock, self.right_dock, self.bottom_dock
                    )
                }>
                    <div class="waw-taskbar"/>
                    <div class="waw-top-dock"/>
                    <div class="waw-left-dock"/>
                    <div class="waw-bottom-dock"/>
                    <div class="waw-right-dock"/>
                    <div class="waw-center-dock">
                        {format!("Your screen is {} by {} pixel", self.width, self.height)}
                    </div>
                </div>
                {for windows}
            </>
        };
    }
}
