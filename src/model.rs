use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::SplitWhitespace;

use crate::geometry::{Vec2f, Vec3f};

pub struct Model {
    verts: Vec<Vec3f>,
    #[allow(dead_code)]
    uvs: Vec<Vec2f>,
    #[allow(dead_code)]
    normals: Vec<Vec3f>,
    faces: Vec<Vec<usize>>,
}

impl Model {
    pub fn new(filename: &str) -> Result<Self, String> {
        let mut verts: Vec<Vec3f> = Vec::new();
        let mut uvs: Vec<Vec2f> = Vec::new();
        let mut normals: Vec<Vec3f> = Vec::new();
        let mut faces: Vec<Vec<usize>> = Vec::new();

        let Ok(file) = File::open(&Path::new(filename)) else {
            return Err("Failed to open file".to_string());
        };
        let reader = BufReader::new(file);

        let parse_coordinate = |parts: &mut SplitWhitespace<'_>, message| {
            parts
                .next()
                .ok_or("Missing coordinate")?
                .parse::<f32>()
                .map_err(|_| message)
        };

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(line) => line,
                Err(e) => return Err(e.to_string()),
            };

            match line.split_whitespace().next() {
                Some("v") => {
                    let mut parts = line[2..].split_whitespace();
                    let x = parse_coordinate(&mut parts, "Failed to parse x coordinate")?;
                    let y = parse_coordinate(&mut parts, "Failed to parse y coordinate")?;
                    let z = parse_coordinate(&mut parts, "Failed to parse x coordinate")?;
                    verts.push(Vec3f::new(x, y, z));
                }
                Some("f") => {
                    let mut face = Vec::new();
                    let parts = line[2..].split_whitespace();
                    for part in parts {
                        let indices: Vec<&str> = part.split('/').collect();
                        if indices.is_empty() {
                            return Err(format!("No vertex index found in part: {}", part));
                        }
                        if let Ok(v_idx) = indices[0].parse::<usize>() {
                            face.push(v_idx - 1); // OBJ index starts from 1
                        } else {
                            return Err(format!("Failed to parse vertex index: {}", indices[0]));
                        }
                    }
                    faces.push(face);
                }
                Some("vt") => {
                    let mut parts = line[2..].split_whitespace();
                    let u = parse_coordinate(&mut parts, "Failed to parse u coordinate")?;
                    let v = parse_coordinate(&mut parts, "Failed to parse v coordinate")?;
                    uvs.push(Vec2f::new(u, v));
                }
                Some("vn") => {
                    let mut parts = line[2..].split_whitespace();
                    let nx = parse_coordinate(&mut parts, "Failed to parse nx coordinate")?;
                    let ny = parse_coordinate(&mut parts, "Failed to parse ny coordinate")?;
                    let nz = parse_coordinate(&mut parts, "Failed to parse nx coordinate")?;
                    normals.push(Vec3f::new(nx, ny, nz));
                }
                Some(&_) => continue,
                None => continue,
            };
        }

        Ok(Model {
            verts,
            uvs,
            normals,
            faces,
        })
    }

    #[allow(dead_code)]
    pub fn nverts(&self) -> usize {
        self.verts.len()
    }

    pub fn nfaces(&self) -> usize {
        self.faces.len()
    }

    pub fn vert(&self, idx: usize) -> Vec3f {
        self.verts[idx]
    }

    pub fn face(&self, idx: usize) -> &Vec<usize> {
        &self.faces[idx]
    }
}
