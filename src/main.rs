mod geometry;
mod model;
mod renderer;
mod tgaimage;

use model::Model;
use renderer::Renderer;
use tgaimage::{Format, TGAImage};

fn main() {
    let width = 800;
    let height = 800;
    let model = match Model::new("src/obj/african_head.obj") {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Failed to load model: {}", e);
            std::process::exit(1);
        }
    };

    let mut renderer = Renderer::new(width, height);
    let mut texture_image = TGAImage::new(0, 0, Format::RGB);
    texture_image
        .read_tga_file("src/texture/african_head_diffuse.tga")
        .expect("Failed to load texture image");
    texture_image.flip_vertically();
    renderer.render_model(&model, &texture_image).unwrap();
    renderer.save_tga_image("output.tga").unwrap();
}
