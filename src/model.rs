use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::geometry::Vec3f;

pub struct Model {
    verts: Vec<Vec3f>,
    faces: Vec<Vec<usize>>,
}

impl Model {
    pub fn new(filename: &str) -> Self {
        let mut verts: Vec<Vec3f> = Vec::new();
        let mut faces: Vec<Vec<usize>> = Vec::new();

        let file = File::open(&Path::new(filename)).expect("Failed to open file");
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                if line.starts_with("v ") {
                    let mut parts = line[2..].split_whitespace();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    verts.push(Vec3f::new(x, y, z));
                } else if line.starts_with("f ") {
                    let mut face = Vec::new();
                    let parts = line[2..].split_whitespace();
                    for part in parts {
                        let indices: Vec<&str> = part.split('/').collect();
                        if !indices.is_empty() {
                            if let Ok(v_idx) = indices[0].parse::<usize>() {
                                face.push(v_idx - 1); // OBJ index starts from 1
                            } else {
                                eprintln!("Failed to parse vertex index: {}", indices[0]);
                                continue;
                            }
                        } else {
                            eprintln!("No vertex index found in part: {}", part);
                            continue;
                        }
                    }
                    faces.push(face);
                }
            }
        }

        eprintln!("# v# {} f# {}", verts.len(), faces.len());

        Model { verts, faces }
    }

    #[allow(dead_code)]
    pub fn nverts(&self) -> usize {
        self.verts.len()
    }

    pub fn nfaces(&self) -> usize {
        self.faces.len()
    }

    pub fn vert(&self, i: usize) -> Vec3f {
        self.verts[i]
    }

    pub fn face(&self, idx: usize) -> &Vec<usize> {
        &self.faces[idx]
    }
}
