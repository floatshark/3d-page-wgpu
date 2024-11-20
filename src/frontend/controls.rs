use crate::engine::define;

use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
pub struct MouseEventResponseJs {
    pub movement_x: i32,
    pub movement_y: i32,
    pub on_click: bool,
}

pub fn add_event_listener_control(
    view_record: &std::rc::Rc<std::cell::Cell<MouseEventResponseJs>>,
) {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let view_record_clone: std::rc::Rc<std::cell::Cell<MouseEventResponseJs>> = view_record.clone();

    let mouse_move_closure: wasm_bindgen::prelude::Closure<dyn FnMut(_)> =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let record: MouseEventResponseJs = MouseEventResponseJs {
                movement_x: event.movement_x(),
                movement_y: event.movement_y(),
                on_click: event.buttons() == 1,
            };
            view_record_clone.set(record);
        }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("mousemove", mouse_move_closure.as_ref().unchecked_ref())
        .unwrap();
    mouse_move_closure.forget();
}
