/**
 * HTML's drag and drop api worked fine,
 * but it is not mobile friendly.
 */
use std::rc::{Rc, Weak};
use std::sync::Mutex;
use wasm_bindgen::JsCast;
use web_sys::{Event, EventTarget, PointerEvent, MouseEvent, Touch, TouchEvent};
use gloo::utils::window;
use gloo::events::{EventListener, EventListenerOptions};
use yew::prelude::*;

pub trait DragHandler: Sized + 'static {
    fn on_down(&mut self, event: &UnifiedPointerEvent);
    fn on_move(&mut self, event: &UnifiedPointerEvent);
    fn on_up(&mut self, event: &UnifiedPointerEvent);

    fn into_callbacks(self) -> DragCallbacks {
        let outer_drag = Rc::new(Mutex::new(InnerDragHandler {
            listener: None,
            handler: self,
            self_ref: Weak::new(),
        }));
        let mut drag = outer_drag.lock()
            .expect("Nobody else can own this yet.");
        drag.self_ref = Rc::downgrade(&outer_drag);

        DragCallbacks {
            mouse: {
                let handler = drag.get_down_handler();
                Callback::from(move |event: MouseEvent| handler(&event.dyn_ref().unwrap()))
            },
            pointer: {
                let handler = drag.get_down_handler();
                Callback::from(move |event: PointerEvent| handler(&event.dyn_ref().unwrap()))
            },
            touch: {
                let handler = drag.get_down_handler();
                Callback::from(move |event: TouchEvent| handler(&event.dyn_ref().unwrap()))
            },
        }
    }
}

pub struct DragCallbacks {
    pub mouse: Callback<MouseEvent>,
    pub pointer: Callback<PointerEvent>,
    pub touch: Callback<TouchEvent>,
}

struct InnerDragHandler<H: DragHandler> {
    listener: Option<(EventListener, EventListener)>,
    handler: H,
    self_ref: Weak<Mutex<InnerDragHandler<H>>>,
}
impl <H: DragHandler> InnerDragHandler<H> {
    fn get_down_handler(&self) -> impl Fn(&Event) {
        let drag = self.self_ref.upgrade()
            .expect("Self will only exists while this ref is valid");
        move |event: &Event| {
            let event = event.try_into()
                .expect("Listener only registered for supported events");

            let mut drag = drag.lock()
                .expect("No threads involved");

            let target: EventTarget = window()
                .dyn_into()
                .expect("window is an EventTarget");

            drag.listener = Some((
                EventListener::new_with_options(
                    &target,
                    match &event {
                        UnifiedPointerEvent::Mouse(_) => "mousemove",
                        UnifiedPointerEvent::Touch(_) => "touchmove",
                        UnifiedPointerEvent::Pointer(_) => "pointermove",
                    },
                    EventListenerOptions::enable_prevent_default(),
                    drag.get_move_handler()
                ),
                EventListener::new(
                    &target,
                    match &event {
                        UnifiedPointerEvent::Mouse(_) => "mouseup",
                        UnifiedPointerEvent::Touch(_) => "touchend",
                        UnifiedPointerEvent::Pointer(_) => "pointerup",
                    },
                    drag.get_up_handler()
                ),
            ));

            drag.handler.on_down(&event);
        }
    }

    fn get_move_handler(&self) -> impl Fn(&Event) {
        let drag = self.self_ref.upgrade()
            .expect("Self will only exists while this ref is valid");
        move |event: &Event| {
            // Stop mobile browsers from scrolling
            event.prevent_default();

            let event: UnifiedPointerEvent = event.try_into()
                .expect("Listener only registered for supported events");

            let mut drag = drag.lock()
                .expect("No threads involved™");

            /*if event.buttons() == 0 {
                drag.handler.on_up(&event);
                drag.listener = None;
            } else {
                drag.handler.on_move(&event);
            }*/
            drag.handler.on_move(&event);
        }
    }

    fn get_up_handler(&self) -> impl Fn(&Event) {
        let drag = self.self_ref.upgrade()
            .expect("Self will only exists while this ref is valid");
        move |event: &Event| {
            let event: UnifiedPointerEvent = event.try_into()
                .expect("Listener only registered for supported events");

            let mut drag = drag.lock()
                .expect("No threads involved™");

            drag.handler.on_move(&event);
            drag.handler.on_up(&event);
            drag.listener = None;
        }
    }
}

pub enum UnifiedPointerEvent {
    Mouse(MouseEvent),
    Touch(Touch),
    Pointer(PointerEvent),
}
impl From<&MouseEvent> for UnifiedPointerEvent {
    fn from(e: &MouseEvent) -> Self {
        UnifiedPointerEvent::Mouse(e.clone())
    }
}
impl From<&TouchEvent> for UnifiedPointerEvent {
    fn from(e: &TouchEvent) -> Self {
        UnifiedPointerEvent::Touch(e.changed_touches().get(0)
            .expect("At least one touch should have changed"))
    }
}
impl From<&PointerEvent> for UnifiedPointerEvent {
    fn from(e: &PointerEvent) -> Self {
        UnifiedPointerEvent::Pointer(e.clone())
    }
}
impl <'a> TryFrom<&'a Event> for UnifiedPointerEvent {
    type Error = &'a Event;

    fn try_from(e: &'a Event) -> Result<Self, Self::Error> {
        if let Some(mouse) = e.dyn_ref::<MouseEvent>() {
            return Ok(mouse.into());
        }
        if let Some(touch) = e.dyn_ref::<TouchEvent>() {
            return Ok(touch.into());
        }
        if let Some(pointer) = e.dyn_ref::<PointerEvent>() {
            return Ok(pointer.into());
        }
        return Err(e);
    }
}
macro_rules! forward_method {
    ($method:ident, $ret:ty) => {
         pub fn $method(&self) -> $ret {
             match self {
                 UnifiedPointerEvent::Mouse(e)   => e.$method(),
                 UnifiedPointerEvent::Touch(e)   => e.$method(),
                 UnifiedPointerEvent::Pointer(e) => e.$method(),
             }
         }
    }
}
impl UnifiedPointerEvent {
    forward_method!(client_x, i32);
    forward_method!(client_y, i32);
    forward_method!(screen_x, i32);
    forward_method!(screen_y, i32);
    forward_method!(page_x, i32);
    forward_method!(page_y, i32);
    forward_method!(target, Option<EventTarget>);
}
