mod geometry;
mod model;
mod renderer;
mod tgaimage;

use model::Model;
use renderer::Renderer;

fn main() {
    let width = 800;
    let height = 800;
    let model = Model::new("src/obj/african_head.obj");

    let mut renderer = Renderer::new(width, height);
    renderer.render_model(&model);
    renderer.save_tga_image("output.tga");

    let mut renderer2 = Renderer::new(width, height);
    renderer2.render_model2(&model);
    renderer2.save_tga_image("output2.tga");
}
