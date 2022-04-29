use yew::{prelude::*, html};

#[derive(Copy, Clone, PartialEq)]
pub struct WindowId(u32);
impl ToString for WindowId {
    fn to_string(&self) -> String {
        format!("waw-window-{}", self.0)
    }
}
impl From<u32> for WindowId {
    fn from(id: u32) -> Self {
        WindowId(id)
    }
}

#[derive(Copy, Clone, Properties, PartialEq)]
pub struct WindowProps {
    pub id: WindowId,
}
pub struct Window {}
pub enum WindowMsg {}

impl Component for Window {
    type Message = WindowMsg;
    type Properties = WindowProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Window {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        return html! {
            <div id={ctx.props().id.to_string()}>
                {format!("Hello Window {}", ctx.props().id.to_string())}
            </div>
        };
    }
}