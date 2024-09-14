mod tgaimage;
use tgaimage::{Format, TGAColor, TGAImage};

fn main() {
    //let white = TGAColor::rgba(255, 255, 255, 255);
    let red = TGAColor::rgba(255, 0, 0, 255);

    let mut image = TGAImage::new(100, 100, Format::RGB);
    image.set(52, 41, &red);
    image.flip_vertically();
    image
        .write_tga_file("output.tga", true)
        .expect("Failed to write TGA file");
}
