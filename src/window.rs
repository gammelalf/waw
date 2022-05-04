use gloo::utils::document;
use once_cell::unsync::Lazy;
use web_sys::Element;
use yew::prelude::*;
use crate::screen::DockPosition;

pub struct Window {
    pub title: String,
    pub icon: String,
    pub div: Lazy<Element>,
    pub dock: Option<DockPosition>,
    pub hidden: bool,
}
impl Window {
    pub fn new(title: String, icon: String) -> Window {
        Window {
            title, icon,
            div: Lazy::new(||
                document()
                    .create_element("div")
                    .expect("Couldn't create new <div>")
            ),
            dock: None,
            hidden: true,
        }
    }
    pub fn inside_dock(&self) -> Html {
        Html::VRef(self.div.clone().into())
    }
    pub fn inside_taskbar(&self) -> Html {
        return html!{
            <img src={self.icon.clone()} alt={self.title.clone()}/>
        };
    }
}

/*
#[wasm_bindgen(js_name="Window")]
pub struct JsWindow(pub Rc<Window>);
#[wasm_bindgen(js_class="Window")]
impl JsWindow {
    #[wasm_bindgen(getter)]
    pub fn icon(&self) -> String {
        self.0.icon.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.0.title.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn div(&self) -> Element {
        self.0.div.clone()
    }
}
impl From<Rc<Window>> for JsWindow {
    fn from(window: Rc<Window>) -> Self {
        JsWindow(window)
    }
}
*/