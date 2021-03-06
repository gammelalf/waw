use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

pub mod dragndrop;
pub mod promise;
pub mod anchor;
pub mod drop_zone;
pub mod floating;
pub mod window;
pub mod screen;

use crate::promise::{Promise, PendingPromise};
use crate::screen::{Screen, ScreenMsg, ScreenProps};


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
    pub fn new_window(&self, options: JsValue) -> Promise {
        let (promise, pending) = PendingPromise::new();
        if let Ok(init) = options.try_into() {
            self.0.send_message(ScreenMsg::NewWindow(pending, init));
        } else {
            pending.reject("Invalid argument");
        }
        promise
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