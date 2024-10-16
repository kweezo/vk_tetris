use vrtacnik_engine::{self, Renderer};

fn main(){
    let mut window = vrtacnik_engine::Window::new(1280, 720, String::from("ayyy"), false)
    .unwrap();

    let mut renderer = vrtacnik_engine::create_renderer(vrtacnik_engine::Graphics_API::OPENGL, &mut window);

    renderer.set_clear_color(glm::Vector3::new(0f32, 0f32, 0f32));

    while !window.should_close(){

        renderer.render(&mut window);

        window.poll_events();
    }
}