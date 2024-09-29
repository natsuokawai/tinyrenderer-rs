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
    faces: Vec<Vec<Vec<usize>>>,
}

impl Model {
    pub fn new(filename: &str) -> Result<Self, String> {
        let mut verts: Vec<Vec3f> = Vec::new();
        let mut uvs: Vec<Vec2f> = Vec::new();
        let mut normals: Vec<Vec3f> = Vec::new();
        let mut faces: Vec<Vec<Vec<usize>>> = Vec::new();

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
                        let mut idxs = Vec::new();
                        for idx in part.split('/') {
                            match idx.parse::<usize>() {
                                Ok(idx) => idxs.push(idx - 1), // OBJ index starts from 1
                                Err(e) => return Err(e.to_string()),
                            }
                        }
                        face.push(idxs);
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

        let model = Model {
            verts,
            uvs,
            normals,
            faces,
        };

        println!(
            "Model loaded. verts: {}, uvs: {}, normals: {}, faces: {}",
            model.verts.len(),
            model.uvs.len(),
            model.normals.len(),
            model.faces.len()
        );

        Ok(model)
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

    pub fn uv(&self, idx: usize) -> Vec2f {
        self.uvs[idx]
    }

    pub fn normal(&self, idx: usize) -> Vec3f {
        self.normals[idx]
    }

    pub fn face(&self, idx: usize) -> &Vec<Vec<usize>> {
        &self.faces[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model() {
        let model = Model::new("tests/models/sample.obj").expect("Failed to load model.");

        // In the object file form, 1/1/1/2/1 3/3/1, but since the index is subtracted by 1 when storing the data, the assertion is made with the value of 1 subtracted.
        assert_eq!(
            model.faces[0],
            vec![vec![0, 0, 0], vec![1, 1, 0], vec![2, 2, 0]]
        );
    }
}
