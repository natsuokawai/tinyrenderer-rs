use crate::{
    geometry::{Vec2f, Vec2i, Vec3f, Vec3i},
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

    pub fn render_model(
        &mut self,
        model: &crate::model::Model,
        texture_image: &TGAImage,
    ) -> Result<(), String> {
        let light_dir = Vec3f::new(0.0, 0.0, -1.0);
        let mut zbuffer = vec![
            vec![i32::min_value(); self.image.width as usize + 1];
            self.image.height as usize + 1
        ];

        for i in 0..model.nfaces() {
            let face = model.face(i);
            let mut screen_coords = vec![Vec3i::new(0, 0, 0); 3];
            let mut world_coords = vec![Vec3f::new(0.0, 0.0, 0.0); 3];
            let mut texture_coords = vec![Vec2f::new(0.0, 0.0); 3];
            for j in 0..3 {
                let v = model.vert(face[j][0]);
                screen_coords[j] = Vec3i::new(
                    ((v.x + 1.0) * self.width as f32 / 2.0) as i32,
                    ((v.y + 1.0) * self.height as f32 / 2.0) as i32,
                    (v.z * 1000.0) as i32,
                );
                world_coords[j] = v;
                texture_coords[j] = model.uv(face[j][1]);
            }
            let mut n =
                (world_coords[2] - world_coords[0]).cross(world_coords[1] - world_coords[0]);
            n.normalize(1.0);
            let intensity = n.dot(light_dir);
            if intensity > 0.0 {
                self.draw_triangle(
                    screen_coords[0],
                    screen_coords[1],
                    screen_coords[2],
                    texture_coords[0],
                    texture_coords[1],
                    texture_coords[2],
                    texture_image,
                    intensity,
                    &mut zbuffer,
                )?;
            }
        }

        Ok(())
    }

    fn draw_triangle(
        &mut self,
        mut t0: Vec3i,
        mut t1: Vec3i,
        mut t2: Vec3i,
        mut uv0: Vec2f,
        mut uv1: Vec2f,
        mut uv2: Vec2f,
        texture_image: &TGAImage,
        intensity: f32,
        zbuffer: &mut Vec<Vec<i32>>,
    ) -> Result<(), String> {
        if t0.y == t1.y && t0.y == t2.y {
            // Degenerate triangle
            return Ok(());
        }

        let image = &mut self.image;

        // Sort the vertices by y-coordinate ascending (t0.y <= t1.y <= t2.y)
        if t0.y > t1.y {
            std::mem::swap(&mut t0, &mut t1);
            std::mem::swap(&mut uv0, &mut uv1);
        }
        if t0.y > t2.y {
            std::mem::swap(&mut t0, &mut t2);
            std::mem::swap(&mut uv0, &mut uv2);
        }
        if t1.y > t2.y {
            std::mem::swap(&mut t1, &mut t2);
            std::mem::swap(&mut uv1, &mut uv2);
        }

        let total_height = t2.y - t0.y;
        if total_height == 0 {
            return Err("DivisionByZero".to_string());
        }

        for i in 0..total_height {
            let second_half = i > t1.y - t0.y || t1.y == t0.y;
            let segment_height = if second_half {
                t2.y - t1.y
            } else {
                t1.y - t0.y
            };
            let alpha = i as f32 / total_height as f32;
            let beta =
                (i - if second_half { t1.y - t0.y } else { 0 }) as f32 / segment_height as f32;
            let mut p_a = t0.to_f() + (t2.to_f() - t0.to_f()) * alpha;
            let mut p_b = if second_half {
                t1.to_f() + (t2.to_f() - t1.to_f()) * beta
            } else {
                t0.to_f() + (t1.to_f() - t0.to_f()) * beta
            };
            let mut uvp_a = uv0 + (uv2 - uv0) * alpha;
            let mut uvp_b = if second_half {
                uv1 + (uv2 - uv1) * beta
            } else {
                uv0 + (uv1 - uv0) * beta
            };

            if p_a.x > p_b.x {
                std::mem::swap(&mut p_a, &mut p_b);
                std::mem::swap(&mut uvp_a, &mut uvp_b);
            }

            for j in (p_a.x as i32)..=(p_b.x as i32) {
                let phi = if p_b.x as i32 == p_a.x as i32 {
                    1.0
                } else {
                    (j as f32 - p_a.x) / (p_b.x - p_a.x)
                };
                let p_cur = p_a + (p_b - p_a) * phi;
                let uvp_cur = uvp_a + (uvp_b - uvp_a) * phi;

                if zbuffer[p_cur.x as usize][p_cur.y as usize] < p_cur.z as i32 {
                    zbuffer[p_cur.x as usize][p_cur.y as usize] = p_cur.z as i32;
                    let color = match texture_image.get(
                        (uvp_cur.x.abs() * texture_image.width as f32) as i32,
                        (uvp_cur.y.abs() * texture_image.height as f32) as i32,
                    ) {
                        Some(c) => c,
                        None => {
                            return Err(format!(
                                "Texture not found. p_cur: {}, uvp_cur: {}",
                                p_cur, uvp_cur
                            ))
                        }
                    };
                    let [b, g, r, a] = color.raw;
                    image.set(
                        p_cur.x as i32,
                        p_cur.y as i32,
                        &TGAColor::rgba(
                            (r as f32 * intensity) as u8,
                            (g as f32 * intensity) as u8,
                            (b as f32 * intensity) as u8,
                            a,
                        ),
                    );
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
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
