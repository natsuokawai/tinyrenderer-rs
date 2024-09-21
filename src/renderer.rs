use crate::{
    geometry::{Vec2i, Vec3f, Vec3i},
    tgaimage::{Format, TGAColor, TGAImage},
};

pub struct Renderer {
    width: i32,
    height: i32,
    image: TGAImage,
}

#[allow(dead_code)]
pub enum OptimizationLevel {
    Level0,
    Level1,
    Level2,
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Self {
        let image = TGAImage::new(width, height, Format::RGB);
        Renderer {
            width,
            height,
            image,
        }
    }

    pub fn save_tga_image(&mut self, filename: &str) -> std::io::Result<()> {
        self.image.flip_vertically();
        self.image.write_tga_file(filename, true)
    }

    pub fn render_model(&mut self, model: &crate::model::Model) {
        let white = TGAColor::rgba(255, 255, 255, 255);

        for i in 0..model.nfaces() {
            let face = model.face(i);
            for j in 0..3 {
                let v0 = model.vert(face[j][0]);
                let v1 = model.vert(face[(j + 1) % 3][0]);
                let x0 = (v0.x + 1.0) * self.width as f32 / 2.0;
                let y0 = (v0.y + 1.0) * self.height as f32 / 2.0;
                let x1 = (v1.x + 1.0) * self.width as f32 / 2.0;
                let y1 = (v1.y + 1.0) * self.height as f32 / 2.0;
                let t0 = Vec2i::new(x0 as i32, y0 as i32);
                let t1 = Vec2i::new(x1 as i32, y1 as i32);
                self.draw_line(t0, t1, &white, OptimizationLevel::Level0);
            }
        }
    }

    pub fn render_model2(&mut self, model: &crate::model::Model) {
        let light_dir = Vec3f::new(0.0, 0.0, -1.0);
        let mut zbuffer = vec![
            vec![i32::min_value(); self.image.width as usize + 1];
            self.image.height as usize + 1
        ];

        for i in 0..model.nfaces() {
            let face = model.face(i);
            let mut screen_coords = vec![Vec3i::new(0, 0, 0); 3];
            let mut world_coords = vec![Vec3f::new(0.0, 0.0, 0.0); 3];
            for j in 0..3 {
                let v = model.vert(face[j][0]);
                screen_coords[j] = Vec3i::new(
                    ((v.x + 1.0) * self.width as f32 / 2.0) as i32,
                    ((v.y + 1.0) * self.height as f32 / 2.0) as i32,
                    v.z as i32,
                );
                world_coords[j] = v;
            }
            let mut n =
                (world_coords[2] - world_coords[0]).cross(world_coords[1] - world_coords[0]);
            n.normalize(1.0);
            let intensity = n.dot(light_dir);
            if intensity > 0.0 {
                self.draw_triangle_with_zbuffer(
                    screen_coords[0],
                    screen_coords[1],
                    screen_coords[2],
                    &TGAColor::rgba(
                        (intensity * 255.0) as u8,
                        (intensity * 255.0) as u8,
                        (intensity * 255.0) as u8,
                        255,
                    ),
                    &mut zbuffer,
                );
            }
        }
    }

    pub fn draw_triangle_with_zbuffer(
        &mut self,
        t0: Vec3i,
        t1: Vec3i,
        t2: Vec3i,
        color: &TGAColor,
        zbuffer: &mut Vec<Vec<i32>>,
    ) {
        let image = &mut self.image;

        let bbox_min_x = std::cmp::max(0, *vec![t0.x, t1.x, t2.x].iter().min().unwrap());
        let bbox_max_x = std::cmp::min(image.width, *vec![t0.x, t1.x, t2.x].iter().max().unwrap());
        let bbox_min_y = std::cmp::max(0, *vec![t0.y, t1.y, t2.y].iter().min().unwrap());
        let bbox_max_y = std::cmp::min(image.height, *vec![t0.y, t1.y, t2.y].iter().max().unwrap());

        for x in bbox_min_x..=bbox_max_x {
            for y in bbox_min_y..=bbox_max_y {
                let bc = Self::barycentric(
                    Vec2i::new(t0.x, t0.y),
                    Vec2i::new(t1.x, t1.y),
                    Vec2i::new(t2.x, t2.y),
                    Vec2i::new(x, y),
                );
                if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                    continue;
                }
                let z = t0.x * bc.x as i32 + t1.y * bc.y as i32 + t2.z * bc.z as i32;
                if z > zbuffer[x as usize][y as usize] {
                    zbuffer[x as usize][y as usize] = z;
                    image.set(x, y, color);
                }
            }
        }
    }

    fn barycentric(a: Vec2i, b: Vec2i, c: Vec2i, p: Vec2i) -> Vec3f {
        let u = Vec3i::new(b.x - a.x, c.x - a.x, a.x - p.x).cross(Vec3i::new(
            b.y - a.y,
            c.y - a.y,
            a.y - p.y,
        ));
        if u.z.abs() < 1 {
            return Vec3f::new(-1.0, 1.0, 1.0);
        }
        Vec3f::new(
            1.0 - (u.x + u.y) as f32 / u.z as f32,
            u.y as f32 / u.z as f32,
            u.x as f32 / u.z as f32,
        )
    }

    pub fn draw_line(
        &mut self,
        t0: Vec2i,
        t1: Vec2i,
        color: &TGAColor,
        optimization_level: OptimizationLevel,
    ) {
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

        match optimization_level {
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
            let mut renderer = Renderer::new(width, height);
            renderer.draw_line(
                Vec2i::new(0, 0),
                Vec2i::new(8, 5),
                c,
                test.optimization_level,
            );

            let mut testimage = TGAImage::new(width, height, Format::RGB);
            testimage.read_tga_file(test.filename).unwrap();
            testimage.flip_vertically();

            assert_eq!(renderer.image.data, testimage.data);
        }
    }
}
