mod geometry;
mod model;
mod renderer;
mod tgaimage;

use geometry::Vec2i;
use model::Model;
use renderer::Renderer;
use tgaimage::TGAColor;

fn main() {
    let width = 800;
    let height = 800;
    let model = Model::new("src/obj/african_head.obj");

    let mut renderer = Renderer::new(width, height, renderer::OptimizationLevel::Level0);
    renderer.render_model(&model);
    renderer.save_tga_image("output.tga");

    let mut triangle_renderer = Renderer::new(200, 200, renderer::OptimizationLevel::Level0);
    triangle_renderer.draw_triangle(
        Vec2i::new(10, 70),
        Vec2i::new(50, 160),
        Vec2i::new(70, 80),
        &TGAColor::rgba(255, 0, 0, 255),
    );
    triangle_renderer.draw_triangle(
        Vec2i::new(180, 50),
        Vec2i::new(150, 10),
        Vec2i::new(70, 180),
        &TGAColor::rgba(255, 255, 255, 255),
    );
    triangle_renderer.draw_triangle(
        Vec2i::new(180, 150),
        Vec2i::new(120, 160),
        Vec2i::new(130, 180),
        &TGAColor::rgba(0, 255, 0, 255),
    );
    triangle_renderer.save_tga_image("triangle.tga");
}
