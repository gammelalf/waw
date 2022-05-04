use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DropZoneProps {
    #[prop_or_default] pub class: Classes,
    #[prop_or_default] pub over_class: Classes,
}
pub struct DropZone {
    pub over: bool,
}
impl Component for DropZone {
    type Message = bool;
    type Properties = DropZoneProps;

    fn create(_ctx: &Context<Self>) -> Self {
        DropZone {
            over: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.over = msg;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        return html!{
            <div
                class={
                    if self.over {
                        classes!(props.class.clone(), props.over_class.clone())
                    } else {
                        classes!(props.class.clone())
                    }
                }
                ondragenter={ctx.link().callback(|_| true)}
                ondragleave={ctx.link().callback(|_| false)}
            />
        };
    }
}