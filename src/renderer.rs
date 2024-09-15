use crate::{
    geometry::Vec2i,
    tgaimage::{Format, TGAColor, TGAImage},
};

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
                let t0 = Vec2i::new(x0 as i32, y0 as i32);
                let t1 = Vec2i::new(x1 as i32, y1 as i32);
                self.draw_line(t0, t1, &white);
            }
        }
    }

    pub fn draw_line(&mut self, t0: Vec2i, t1: Vec2i, color: &TGAColor) {
        let image = &mut self.image;
        let mut steep = false;
        let mut x0 = t0.x;
        let mut x1 = t1.x;
        let mut y0 = t0.y;
        let mut y1 = t1.y;

        if (x0 - x1).abs() < (y0 - y1).abs() {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
            steep = true;
        }

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        match self.optimization_level {
            OptimizationLevel::Level0 => {
                for x in x0..=x1 {
                    let t = (x - x0) as f32 / (x1 - x0) as f32;
                    let y = y0 as f32 * (1.0 - t) + y1 as f32 * t;
                    if steep {
                        image.set(y as i32, x, color);
                    } else {
                        image.set(x, y as i32, color);
                    }
                }
            }
            OptimizationLevel::Level1 => {
                let dx = x1 - x0;
                let dy = y1 - y0;
                let derror = dy as f32 / dx as f32;
                let mut error: f32 = 0.0;
                let mut y = y0;
                for x in x0..=x1 {
                    if steep {
                        image.set(y, x, color);
                    } else {
                        image.set(x, y, color);
                    }
                    error += derror;
                    if error > 0.5 {
                        y += if y1 > y0 { 1 } else { -1 };
                        error -= 1.0;
                    }
                }
            }
            OptimizationLevel::Level2 => {
                let dx = x1 - x0;
                let dy = y1 - y0;
                let derror = dy.abs() * 2;
                let mut error = 0;
                let mut y = y0;
                for x in x0..=x1 {
                    if steep {
                        image.set(y, x, color);
                    } else {
                        image.set(x, y, color);
                    }
                    error += derror;
                    if error > dx {
                        y += if y1 > y0 { 1 } else { -1 };
                        error -= dx * 2;
                    }
                }
            }
        }
    }
}
