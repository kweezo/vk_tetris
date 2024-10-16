use vrtacnik_engine;

fn main(){
    let mut window = vrtacnik_engine::Window::new(1280, 720, String::from("ayyy"), false)
    .unwrap();

    let renderer = vrtacnik_engine::create_renderer(vrtacnik_engine::Graphics_API::OPENGL, &mut window);

    while !window.should_close(){

        window.poll_events();
    }
}