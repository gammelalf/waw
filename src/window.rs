use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::Element;
use gloo::utils::document;
use crate::screen::DockPosition;

/**
 * This struct directly matches the javascript object expected `Screen.newWindow`.
 *
 * Json is used as intermediate to pass the initialisation data
 * in a more flexibly manor.
 */
#[derive(Deserialize)]
pub struct WindowInit {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub dock: Option<String>,
    #[serde(alias="requestCenter")]
    pub request_center: Option<bool>,
}
impl TryFrom<JsValue> for WindowInit {
    type Error = serde_json::Error;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        value.into_serde()
    }
}

/**
 * Internal representation for a window
 *
 * Created from a `WindowInit` struct in Screen's update method.
 */
pub struct Window {
    pub title: String,
    pub icon: String,
    pub div: Element,
    pub dock: Option<DockPosition>,
    pub active: bool,
}
impl From<WindowInit> for Window {
    fn from(init: WindowInit) -> Self {
        Window {
            title: init.title.unwrap_or_default(),
            icon: init.icon.unwrap_or_default(),
            div: document()
                .create_element("div")
                .expect("Couldn't create new <div>"),
            dock: init.dock.map(|mut dock| {
                dock.make_ascii_lowercase();
                match &dock[..] {
                    "top"    => Some(DockPosition::Top),
                    "left"   => Some(DockPosition::Left),
                    "bottom" => Some(DockPosition::Bottom),
                    "right"  => Some(DockPosition::Right),
                    _ => None,
                }
            }).flatten(),
            active: false,
        }
    }
}
