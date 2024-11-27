use crate::engine::define;

use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Default)]
pub struct MouseEventResponseJs {
    pub movement_x: i32,
    pub movement_y: i32,
    pub on_click: bool,
    pub wheel_delta_y: f64,
    pub on_wheel: bool,
}

pub fn add_event_listener_control(
    event_response: &std::rc::Rc<std::cell::Cell<MouseEventResponseJs>>,
) {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let event_response_clone_mouse: std::rc::Rc<std::cell::Cell<MouseEventResponseJs>> =
        event_response.clone();

    let mouse_move_closure: wasm_bindgen::prelude::Closure<dyn FnMut(_)> =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut response: MouseEventResponseJs = event_response_clone_mouse.get();

            response.movement_x = event.movement_x();
            response.movement_y = event.movement_y();
            response.on_click = event.which() == 1;

            event_response_clone_mouse.set(response);
        }) as Box<dyn FnMut(_)>);

    let event_response_clone_wheel: std::rc::Rc<std::cell::Cell<MouseEventResponseJs>> =
        event_response.clone();

    let mouse_wheel_closure: wasm_bindgen::prelude::Closure<dyn FnMut(_)> =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
            let mut response: MouseEventResponseJs = event_response_clone_wheel.get();

            response.on_wheel = true;
            response.wheel_delta_y = event.delta_y();

            event_response_clone_wheel.set(response);
        }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("mousemove", mouse_move_closure.as_ref().unchecked_ref())
        .unwrap();
    mouse_move_closure.forget();

    canvas
        .add_event_listener_with_callback("wheel", mouse_wheel_closure.as_ref().unchecked_ref())
        .unwrap();
    mouse_wheel_closure.forget();
}
