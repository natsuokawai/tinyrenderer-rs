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

    pub fn draw_triangle(&mut self, t0_: Vec2i, t1_: Vec2i, t2_: Vec2i, color: &TGAColor) {
        let image = &mut self.image;
        let mut vs = vec![t0_, t1_, t2_];
        vs.sort_by(|a, b| a.y.cmp(&b.y));
        let t0 = vs[0];
        let t1 = vs[1];
        let t2 = vs[2];

        let a01 = (t1.y - t0.y) as f32 / (t1.x - t0.x) as f32;
        let a12 = (t2.y - t1.y) as f32 / (t2.x - t1.x) as f32;
        let a02 = (t2.y - t0.y) as f32 / (t2.x - t0.x) as f32;

        // The equation y = a * (x - p) + q solved for x.
        let calc_x = |y: f32, a: f32, p: f32, q: f32| (y + a * p - q) / a;

        for y in t0.y..=t2.y {
            let mut left_x: f32;
            let mut right_x: f32;
            if y < t1.y {
                left_x = calc_x(y as f32, a02, t0.x as f32, t0.y as f32);
                right_x = calc_x(y as f32, a01, t0.x as f32, t0.y as f32);
            } else {
                left_x = calc_x(y as f32, a02, t0.x as f32, t0.y as f32);
                right_x = calc_x(y as f32, a12, t1.x as f32, t1.y as f32);
            }
            if left_x > right_x {
                std::mem::swap(&mut left_x, &mut right_x);
            }
            for x in (left_x as i32)..=(right_x as i32) {
                image.set(x, y, color);
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
                    let y = y0 as f32 + t * (y1 - y0) as f32;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_line() {
        let width = 10;
        let height = 5;
        let c = &TGAColor::rgba(128, 1, 255, 255);

        struct TestCase<'a> {
            optimization_level: OptimizationLevel,
            filename: &'a str,
        }
        for test in vec![
            TestCase {
                optimization_level: OptimizationLevel::Level0,
                filename: "tests/images/line0.tga",
            },
            TestCase {
                optimization_level: OptimizationLevel::Level1,
                filename: "tests/images/line1.tga",
            },
            TestCase {
                optimization_level: OptimizationLevel::Level2,
                filename: "tests/images/line2.tga",
            },
        ] {
            let mut renderer = Renderer::new(width, height, test.optimization_level);
            renderer.draw_line(Vec2i::new(0, 0), Vec2i::new(8, 5), c);

            let mut testimage = TGAImage::new(width, height, Format::RGB);
            testimage.read_tga_file(test.filename).unwrap();
            testimage.flip_vertically();

            assert_eq!(renderer.image.data, testimage.data);
        }
    }
}
