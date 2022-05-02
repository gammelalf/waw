use gloo::events::EventListener;
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement};
use yew::{prelude::*, html};
use crate::promise::{Promise, PendingPromise};

pub mod window;
pub mod promise;

use crate::window::{Window, WindowId, WindowProps};

#[derive(Properties, PartialEq)]
pub struct ScreenProps {
    parent: HtmlElement,
}
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub resize_listener: EventListener, // Listen on window for size changes
                                        // Listening on direct parent would require ResizeObserver,
                                        // which is unstable and annoying to use in web-sys

    pub windows: Vec<WindowProps>,
    pub next_window_id: u32,
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

                self.windows.push(WindowProps {id: id.into()});
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
                {format!("Your screen is {} by {} pixel", self.width, self.height)}
                {for windows}
            </>
        };
    }
}

#[wasm_bindgen(js_name="Screen")]
pub struct ScreenHandle(AppHandle<Screen>);

#[wasm_bindgen(js_class="Screen")]
impl ScreenHandle {

    #[wasm_bindgen(constructor)]
    pub fn new(parent: HtmlElement) -> ScreenHandle {
        let element: &Element = &parent;
        yew::start_app_with_props_in_element(element.clone(), ScreenProps { parent }).into()
    }

    pub fn resize(&self) {
        self.0.send_message(ScreenMsg::Resize);
    }

    #[wasm_bindgen(js_name="newWindow")]
    pub fn new_window(&self) -> Promise {
        let (promise, pending) = PendingPromise::new();
        self.0.send_message(ScreenMsg::NewWindow(pending));
        promise
    }

    #[wasm_bindgen(js_name="getWindow")]
    pub fn get_window(&self, id: u32) -> Option<Element> {
        gloo::utils::document().get_element_by_id(&WindowId::from(id).to_string())
    }

    #[wasm_bindgen(js_name="deleteWindow")]
    pub fn delete_window(&self, id: u32) {
        self.0.send_message(ScreenMsg::DeleteWindow(id));
    }

    pub fn destroy(self) {
        self.0.destroy();
    }
}
impl From<AppHandle<Screen>> for ScreenHandle {
    fn from(handle: AppHandle<Screen>) -> Self {
        ScreenHandle(handle)
    }
}