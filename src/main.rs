mod engine;
mod frontend;
mod rendering;

use wasm_bindgen::JsCast;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen::prelude::wasm_bindgen(main)]
pub async fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::debug!("Main");

    // Scene
    let mut scene: engine::scene::Scene = engine::scene::Scene::default();
    scene.init();
    let scene: std::rc::Rc<std::cell::RefCell<engine::scene::Scene>> =
        std::rc::Rc::new(std::cell::RefCell::new(scene));

    // Mesh loading
    scene.borrow_mut().objects =
        engine::load::load_gltf_scene(engine::define::GLTF_BATHROOM_PATH).await;

    // Rendering context
    let webgpu_interface: rendering::webgpu::WebGPUInterface =
        rendering::webgpu::init_interface().await;
    let differed_resource: rendering::webgpu::WebGPUDifferedResource =
        rendering::webgpu::init_differed_pipeline(&webgpu_interface);
    rendering::webgpu::init_differed_gbuffer_pipeline(&webgpu_interface, &scene);

    // Javascript Control
    let control_response_js: std::rc::Rc<
        std::cell::RefCell<frontend::eventlistener::ControlResponseJs>,
    > = std::rc::Rc::new(std::cell::RefCell::new(
        frontend::eventlistener::ControlResponseJs::default(),
    ));
    frontend::eventlistener::add_event_listener_control(&control_response_js);

    // Frontend GUI
    frontend::gui::start_gui(&scene);

    // Loop
    let f: std::rc::Rc<_> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g: std::rc::Rc<std::cell::RefCell<Option<_>>> = f.clone();
    *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        engine::scene::update_js(&scene, &control_response_js);

        let render_type = scene.borrow().render_type;

        match render_type {
            0 => {
                rendering::webgpu::update_differed_shading(
                    &webgpu_interface,
                    &scene,
                    &differed_resource,
                );
                rendering::webgpu::render_differed_shading_main(
                    &webgpu_interface,
                    &scene,
                    &differed_resource,
                );
            }
            1 => {
                rendering::webgpu::update_forward_shading(&webgpu_interface, &scene);
                rendering::webgpu::render_forward_shading_main(&webgpu_interface, &scene);
            }
            _ => {}
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    })
        as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());

    log::debug!("Main end");
}

fn request_animation_frame(f: &wasm_bindgen::closure::Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
