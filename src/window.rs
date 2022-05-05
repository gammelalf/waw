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
#[serde(deny_unknown_fields)]
pub struct WindowInit {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub dock: Dock,
    #[serde(alias="requestCenter")]
    pub request_center: Option<bool>,
}
impl TryFrom<JsValue> for WindowInit {
    type Error = serde_json::Error;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        value.into_serde()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dock {
    Top, Left, Bottom, Right
}
impl From<Dock> for DockPosition {
    fn from(dock: Dock) -> Self {
        match dock {
            Dock::Top => DockPosition::Top,
            Dock::Left => DockPosition::Left,
            Dock::Bottom => DockPosition::Bottom,
            Dock::Right => DockPosition::Right,
        }
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
    pub dock: DockPosition,
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
            dock: init.dock.into(),
            active: false,
        }
    }
}
