use crate::tgaimage::{Format, TGAColor, TGAImage};

pub struct Renderer {
    width: i32,
    height: i32,
    image: TGAImage,
    optimization_level: OptimizationLevel,
}

#[allow(dead_code)]
pub enum OptimizationLevel {
    Level0,
    Level1,
    Level2,
}

impl Renderer {
    pub fn new(width: i32, height: i32, optimization_level: OptimizationLevel) -> Self {
        let image = TGAImage::new(width, height, Format::RGB);
        Renderer {
            width,
            height,
            image,
            optimization_level,
        }
    }

    pub fn save_tga_image(&mut self, filename: &str) {
        self.image.flip_vertically();
        self.image
            .write_tga_file(filename, true)
            .expect("Failed to write TGA file");
    }

    pub fn render_model(&mut self, model: &crate::model::Model) {
        let white = TGAColor::rgba(255, 255, 255, 255);

        for i in 0..model.nfaces() {
            let face = model.face(i);
            for j in 0..3 {
                let v0 = model.vert(face[j]);
                let v1 = model.vert(face[(j + 1) % 3]);
                let x0 = (v0.x + 1.0) * self.width as f32 / 2.0;
                let y0 = (v0.y + 1.0) * self.height as f32 / 2.0;
                let x1 = (v1.x + 1.0) * self.width as f32 / 2.0;
                let y1 = (v1.y + 1.0) * self.height as f32 / 2.0;
                self.draw_line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, &white);
            }
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: &TGAColor) {
        let image = &mut self.image;
        let mut steep = false;
        let mut xs = x0;
        let mut xe = x1;
        let mut ys = y0;
        let mut ye = y1;

        if (xs - xe).abs() < (ys - ye).abs() {
            std::mem::swap(&mut xs, &mut ys);
            std::mem::swap(&mut xe, &mut ye);
            steep = true;
        }

        if xs > xe {
            std::mem::swap(&mut xs, &mut xe);
            std::mem::swap(&mut ys, &mut ye);
        }

        match self.optimization_level {
            OptimizationLevel::Level0 => {
                for x in xs..=xe {
                    let t = (x - xs) as f32 / (xe - xs) as f32;
                    let y = ys as f32 * (1.0 - t) + ye as f32 * t;
                    if steep {
                        image.set(y as i32, x, color);
                    } else {
                        image.set(x, y as i32, color);
                    }
                }
            }
            OptimizationLevel::Level1 => {
                let dx = xe - xs;
                let dy = ye - ys;
                let derror = dy as f32 / dx as f32;
                let mut error: f32 = 0.0;
                let mut y = ys;
                for x in xs..=xe {
                    if steep {
                        image.set(y, x, color);
                    } else {
                        image.set(x, y, color);
                    }
                    error += derror;
                    if error > 0.5 {
                        y += if ye > ys { 1 } else { -1 };
                        error -= 1.0;
                    }
                }
            }
            OptimizationLevel::Level2 => {
                let dx = xe - xs;
                let dy = ye - ys;
                let derror = dy.abs() * 2;
                let mut error = 0;
                let mut y = ys;
                for x in xs..=xe {
                    if steep {
                        image.set(y, x, color);
                    } else {
                        image.set(x, y, color);
                    }
                    error += derror;
                    if error > dx {
                        y += if ye > ys { 1 } else { -1 };
                        error -= dx * 2;
                    }
                }
            }
        }
    }
}
