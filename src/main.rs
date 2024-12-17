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

    // Javascript

    let mouse_event_js: std::rc::Rc<
        std::cell::Cell<frontend::eventlistener::MouseEventResponseJs>,
    > = std::rc::Rc::new(std::cell::Cell::new(
        frontend::eventlistener::MouseEventResponseJs::default(),
    ));
    frontend::eventlistener::add_event_listener_control(&mouse_event_js);

    // Scene

    let scene: std::rc::Rc<std::cell::Cell<engine::update::Scene>> =
        std::rc::Rc::new(std::cell::Cell::new(engine::update::Scene::get_init()));

    // GUI

    frontend::gui::start_gui(&scene);

    // Model loading

    let obj_meshes: Vec<rendering::common::Mesh> =
        engine::load::load_obj(engine::define::OBJ_SPHERE_PATH).await;

    // Rendering

    let webgpu_interface: rendering::webgpu::WebGPUInterface =
        rendering::webgpu::init_interface().await;
    let gbuffers: rendering::webgpu::WebGPUDifferedGBuffer =
        rendering::webgpu::init_differed_gbuffer(&webgpu_interface);

    let mut gbuffer_resources: Vec<rendering::webgpu::WebGPURenderResource> = Vec::new();
    for obj_mesh in obj_meshes.iter() {
        let gbuffer_resource: rendering::webgpu::WebGPURenderResource =
            rendering::webgpu::init_differed_gbuffers_shader(&webgpu_interface, &obj_mesh);
        gbuffer_resources.push(gbuffer_resource);
    }
    let differed_resource: rendering::webgpu::WebGPUDifferedResource =
        rendering::webgpu::init_differed_shading(&webgpu_interface, &gbuffers);

    let mut webgpu_resources: Vec<rendering::webgpu::WebGPURenderResource> = Vec::new();
    for obj_mesh in obj_meshes.iter() {
        let webgpu_resource = rendering::webgpu::init_phong_shader(&webgpu_interface, &obj_mesh);
        webgpu_resources.push(webgpu_resource);
    }

    // Loop

    let f: std::rc::Rc<_> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g: std::rc::Rc<std::cell::RefCell<Option<_>>> = f.clone();
    *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        engine::update::update_js(&scene, &mouse_event_js);

        if scene.get().render_type == 0 {
            for gbuffer_resource in gbuffer_resources.iter() {
                rendering::webgpu::upadte_differed_gbuffers_buffer(
                    &scene,
                    &webgpu_interface,
                    &gbuffer_resource,
                );
            }
            rendering::webgpu::update_differed_buffer(
                &scene,
                &webgpu_interface,
                &differed_resource,
            );

            rendering::webgpu::render_differed_main(
                &webgpu_interface,
                &scene,
                &gbuffers,
                &gbuffer_resources,
                &differed_resource,
            );
        } else if scene.get().render_type == 1 {
            for webgpu_resource in webgpu_resources.iter() {
                rendering::webgpu::update_phong_buffer(&scene, &webgpu_interface, &webgpu_resource);
            }
            rendering::webgpu::render_forward_main(&webgpu_interface, &scene, &webgpu_resources);
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
