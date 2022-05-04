use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

pub mod promise;
pub mod anchor;
pub mod floating;
pub mod window;
pub mod screen;

use crate::promise::{Promise, PendingPromise};
use crate::floating::WindowId;
use crate::screen::{DockPosition, Screen, ScreenMsg, ScreenProps};


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
    pub fn new_window(&self, title: String, icon: String) -> Promise {
        let (promise, pending) = PendingPromise::new();
        self.0.send_message(ScreenMsg::NewWindow(pending, title, icon));
        promise
    }

    #[wasm_bindgen(js_name="getWindow")]
    pub fn get_window(&self, id: u32) -> Option<Element> {
        Some(self.0.get_component()?.windows.get(id as usize)?.div.clone())
    }

    pub fn move_window(&self, id: u32, dock: i32) {
        let id = id as usize;
        let dock = match dock {
            0 => Some(DockPosition::Top),
            1 => Some(DockPosition::Left),
            2 => Some(DockPosition::Bottom),
            3 => Some(DockPosition::Right),
            _ => None,
        };
        self.0.send_message(ScreenMsg::MoveWindow(id, dock));
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