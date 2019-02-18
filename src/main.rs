extern crate cgmath;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use cgmath::prelude::*;
use cgmath::Vector3;
use serde::ser::{SerializeSeq, Serializer};
use serde::Serialize;

#[derive(Debug)]
struct Triangle {
    a: usize,
    b: usize,
    c: usize,
}

impl Triangle {
    fn new(a: usize, b: usize, c: usize) -> Triangle {
        Triangle { a, b, c }
    }
}

impl Serialize for Triangle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec_indices = vec![self.a, self.b, self.c];
        let mut seq = serializer.serialize_seq(Some(vec_indices.len()))?;
        for index in vec_indices {
            seq.serialize_element(&index)?;
        }
        seq.end()
    }
}

#[derive(Debug)]
struct ArraySerializedVector(Vector3<f32>);

#[derive(Serialize, Debug)]
struct Polyhedron {
    positions: Vec<ArraySerializedVector>,
    cells: Vec<Triangle>,
}

impl Serialize for ArraySerializedVector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let values = vec![self.0.x, self.0.y, self.0.z];
        let mut seq = serializer.serialize_seq(Some(values.len()))?;
        for value in values {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

impl Polyhedron {
    fn new() -> Polyhedron {
        Polyhedron {
            positions: vec![],
            cells: vec![],
        }
    }

    fn new_isocahedron(radius: f32, detail: usize) -> Polyhedron {
        let t = (1.0 + (5.0 as f32).sqrt()) / 2.0;
        let base_isocahedron = Polyhedron {
            positions: vec![
                ArraySerializedVector(Vector3::new(-1.0, t, 0.0)),
                ArraySerializedVector(Vector3::new(1.0, t, 0.0)),
                ArraySerializedVector(Vector3::new(-1.0, -t, 0.0)),
                ArraySerializedVector(Vector3::new(1.0, -t, 0.0)),
                ArraySerializedVector(Vector3::new(0.0, -1.0, t)),
                ArraySerializedVector(Vector3::new(0.0, 1.0, t)),
                ArraySerializedVector(Vector3::new(0.0, -1.0, -t)),
                ArraySerializedVector(Vector3::new(0.0, 1.0, -t)),
                ArraySerializedVector(Vector3::new(t, 0.0, -1.0)),
                ArraySerializedVector(Vector3::new(t, 0.0, 1.0)),
                ArraySerializedVector(Vector3::new(-t, 0.0, -1.0)),
                ArraySerializedVector(Vector3::new(-t, 0.0, 1.0)),
            ],
            cells: vec![
                Triangle::new(0, 11, 5),
                Triangle::new(0, 5, 1),
                Triangle::new(0, 1, 7),
                Triangle::new(0, 7, 10),
                Triangle::new(0, 10, 11),
                Triangle::new(1, 5, 9),
                Triangle::new(5, 11, 4),
                Triangle::new(11, 10, 2),
                Triangle::new(10, 7, 6),
                Triangle::new(7, 1, 8),
                Triangle::new(3, 9, 4),
                Triangle::new(3, 4, 2),
                Triangle::new(3, 2, 6),
                Triangle::new(3, 6, 8),
                Triangle::new(3, 8, 9),
                Triangle::new(4, 9, 5),
                Triangle::new(2, 4, 11),
                Triangle::new(6, 2, 10),
                Triangle::new(8, 6, 7),
                Triangle::new(9, 8, 1),
            ],
        };
        let mut subdivided = Polyhedron::new();
        subdivided.subdivide(base_isocahedron, radius, detail);
        subdivided
    }

    fn new_dual_isocahedron(radius: f32, detail: usize) -> Polyhedron {
        let isocahedron = Polyhedron::new_isocahedron(radius, detail);
        let mut dual_isocahedron = Polyhedron::new();
        dual_isocahedron.dual(isocahedron);
        dual_isocahedron
    }

    fn subdivide(&mut self, other: Polyhedron, radius: f32, detail: usize) {
        // TODO: maybe make this part of the polygon stuct to avoid having to pass it around
        let mut added_vert_cache: HashMap<(i32, i32, i32), usize> = HashMap::new();
        let precision = 10_f32.powi(4);
        for triangle in other.cells {
            let a = other.positions[triangle.a].0;
            let b = other.positions[triangle.b].0;
            let c = other.positions[triangle.c].0;
            self.subdivide_triangle(a, b, c, radius, detail, precision, &mut added_vert_cache);
        }
    }

    fn subdivide_triangle(
        &mut self,
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
        radius: f32,
        detail: usize,
        precision: f32,
        added_vert_cache: &mut HashMap<(i32, i32, i32), usize>,
    ) {
        let cols = 2usize.pow(detail as u32);
        let mut new_vertices: Vec<Vec<Vector3<f32>>> = vec![];

        for i in 0..=cols {
            new_vertices.push(vec![]);
            let aj = a.clone().lerp(c, i as f32 / cols as f32);
            let bj = b.clone().lerp(c, i as f32 / cols as f32);
            let rows = cols - i;

            for j in 0..=rows {
                if j == 0 && i == cols {
                    new_vertices[i].push(aj.normalize() * radius);
                } else {
                    new_vertices[i]
                        .push(aj.clone().lerp(bj, j as f32 / rows as f32).normalize() * radius);
                }
            }
        }

        for i in 0..cols {
            for j in 0..2 * (cols - i) - 1 {
                let k = j / 2;

                let mut triangle = Triangle { a: 0, b: 0, c: 0 };
                if j % 2 == 0 {
                    triangle.a =
                        self.add_position(new_vertices[i][k + 1], precision, added_vert_cache);
                    triangle.b =
                        self.add_position(new_vertices[i + 1][k], precision, added_vert_cache);
                    triangle.c = self.add_position(new_vertices[i][k], precision, added_vert_cache);
                } else {
                    triangle.a =
                        self.add_position(new_vertices[i][k + 1], precision, added_vert_cache);
                    triangle.b =
                        self.add_position(new_vertices[i + 1][k + 1], precision, added_vert_cache);
                    triangle.c =
                        self.add_position(new_vertices[i + 1][k], precision, added_vert_cache);
                }

                self.cells.push(triangle);
            }
        }
    }

    fn add_position(
        &mut self,
        vertex: Vector3<f32>,
        precision: f32,
        added_vert_cache: &mut HashMap<(i32, i32, i32), usize>,
    ) -> usize {
        let vertex_key = (
            (vertex.x * precision).round() as i32,
            (vertex.y * precision).round() as i32,
            (vertex.z * precision).round() as i32,
        );
        if let Some(added_vert_index) = added_vert_cache.get(&vertex_key) {
            return *added_vert_index;
        } else {
            self.positions.push(ArraySerializedVector(vertex));
            let added_index = self.positions.len() - 1;
            added_vert_cache.insert(vertex_key, added_index);
            return added_index;
        }
    }

    fn dual(&mut self, other: Polyhedron) {
        let vert_to_faces = other.vert_to_faces();
        let original_vert_count = other.positions.len();
        let triangle_centroids = other.triangle_centroids();
        let mut mid_centroid_cache: HashMap<(usize, usize), Vector3<f32>> = HashMap::new();
        let mut hex_count = 0;
        let mut pent_count = 0;
        for i in 0..original_vert_count {
            let faces = &vert_to_faces[&i];
            if faces.len() == 6 {
                hex_count += 1;
            } else {
                pent_count += 1;
            }

            let center_point = find_center_of_triangles(faces, &triangle_centroids);

            for face_index in faces {
                let triangle = &other.cells[*face_index];
                let other_verts: Vec<usize> = vec![triangle.a, triangle.b, triangle.c]
                    .drain(..)
                    .filter(|vert| *vert != i)
                    .collect();
                let sorted_triangle = Triangle::new(i, other_verts[0], other_verts[1]);

                let centroid = triangle_centroids[face_index];
                let mid_b_centroid = other.calculate_mid_centroid(
                    sorted_triangle.b,
                    faces,
                    centroid,
                    &triangle_centroids,
                    &mut mid_centroid_cache,
                );
                let mid_c_centroid = other.calculate_mid_centroid(
                    sorted_triangle.c,
                    faces,
                    centroid,
                    &triangle_centroids,
                    &mut mid_centroid_cache,
                );

                let positions_start = self.positions.len();
                // TODO: remove duplication here:
                // push all triangle_centroids at beginning, same index as face_index
                // push center_point once in outer loop and save index
                // (is it okay if vertices show up in positions out of order like that?)
                // -- yes, I think it is okay
                self.positions.push(ArraySerializedVector(center_point));
                self.positions.push(ArraySerializedVector(centroid));
                self.positions.push(ArraySerializedVector(mid_b_centroid));
                self.positions.push(ArraySerializedVector(mid_c_centroid));

                self.cells.push(Triangle::new(
                    positions_start,
                    positions_start + 1,
                    positions_start + 2,
                ));
                self.cells.push(Triangle::new(
                    positions_start,
                    positions_start + 1,
                    positions_start + 3,
                ));
            }
        }
        println!("hexagons: {}", hex_count);
        println!("pentagons: {}", pent_count);
    }

    fn vert_to_faces(&self) -> HashMap<usize, Vec<usize>> {
        let mut vert_to_faces: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..self.cells.len() {
            let triangle = &self.cells[i];

            if let Some(faces) = vert_to_faces.get_mut(&triangle.a) {
                faces.push(i);
            } else {
                vert_to_faces.insert(triangle.a, vec![i]);
            }

            if let Some(faces) = vert_to_faces.get_mut(&triangle.b) {
                faces.push(i);
            } else {
                vert_to_faces.insert(triangle.b, vec![i]);
            }

            if let Some(faces) = vert_to_faces.get_mut(&triangle.c) {
                faces.push(i);
            } else {
                vert_to_faces.insert(triangle.c, vec![i]);
            }
        }
        vert_to_faces
    }

    fn triangle_centroids(&self) -> HashMap<usize, Vector3<f32>> {
        let mut triangle_centroids: HashMap<usize, Vector3<f32>> = HashMap::new();
        for i in 0..self.cells.len() {
            let a = self.positions[self.cells[i].a].0;
            let b = self.positions[self.cells[i].b].0;
            let c = self.positions[self.cells[i].c].0;
            triangle_centroids.insert(i, calculate_centroid(a, b, c));
        }
        triangle_centroids
    }

    fn calculate_mid_centroid(
        &self,
        vertex_index: usize,
        faces: &Vec<usize>,
        centroid: Vector3<f32>,
        triangle_centroids: &HashMap<usize, Vector3<f32>>,
        mid_centroid_cache: &mut HashMap<(usize, usize), Vector3<f32>>,
    ) -> Vector3<f32> {
        let adj_face_index = self.find_adjacent_face(vertex_index, faces).unwrap();
        let adj_centroid = triangle_centroids[&adj_face_index];
        if let Some(mid_centroid) = mid_centroid_cache.get(&(vertex_index, adj_face_index)) {
            return *mid_centroid;
        } else {
            let mid_centroid = centroid.clone().lerp(adj_centroid, 0.5);
            mid_centroid_cache.insert((vertex_index, adj_face_index), mid_centroid);
            return mid_centroid;
        }
    }

    fn find_adjacent_face(&self, vertex_index: usize, faces: &Vec<usize>) -> Option<usize> {
        for face_index in faces {
            let triangle = &self.cells[*face_index];
            if triangle.a == vertex_index
                || triangle.b == vertex_index
                || triangle.c == vertex_index
            {
                return Some(*face_index);
            }
        }
        None
    }
}

fn calculate_centroid(pa: Vector3<f32>, pb: Vector3<f32>, pc: Vector3<f32>) -> Vector3<f32> {
    let vab_half = (pb.clone() - pa) / 2.0;
    let pab_half = pa.clone() + vab_half;
    ((pc.clone() - pab_half) * (1.0 / 3.0)) + pab_half
}

fn find_center_of_triangles(
    triangle_indices: &Vec<usize>,
    triangle_centroids: &HashMap<usize, Vector3<f32>>,
) -> Vector3<f32> {
    let mut center_point: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
    for triangle_index in triangle_indices.iter() {
        center_point += triangle_centroids[triangle_index];
    }
    center_point /= triangle_indices.len() as f32;
    center_point
}

fn generate_icosahedron_files(dir: &str, param_list: Vec<(f32, usize)>) {
    for param in param_list {
        println!(
            "Generating icosahedron with radius {} and detail {}...",
            param.0, param.1
        );
        let filename = Path::new(dir).join(format!("icosahedron_r{}_d{}.json", param.0, param.1));
        let mut file = File::create(filename).expect("Can't create file");
        let icosahedron = Polyhedron::new_isocahedron(param.0, param.1);
        println!("triangles: {}", icosahedron.cells.len());
        println!("vertices: {}", icosahedron.positions.len());
        let icosahedron_json = serde_json::to_string(&icosahedron).expect("Problem serializing");
        file.write_all(icosahedron_json.as_bytes())
            .expect("Can't write to file");
    }
}

fn generate_hexsphere_files(dir: &str, param_list: Vec<(f32, usize)>) {
    for param in param_list {
        println!(
            "Generating hexsphere with radius {} and detail {}...",
            param.0, param.1
        );
        let filename = Path::new(dir).join(format!("hexsphere_r{}_d{}.json", param.0, param.1));
        let mut file = File::create(filename).expect("Can't create file");
        let hexsphere = Polyhedron::new_dual_isocahedron(param.0, param.1);
        println!("triangles: {}", hexsphere.cells.len());
        println!("vertices: {}", hexsphere.positions.len());
        let hexsphere_json = serde_json::to_string(&hexsphere).expect("Problem serializing");
        file.write_all(hexsphere_json.as_bytes())
            .expect("Can't write to file");
    }
}

fn main() {
    generate_hexsphere_files(
        "output/",
        vec![
            (1.0, 0),
            (1.0, 1),
            (1.0, 2),
            (1.0, 3),
            (1.0, 4),
            (1.0, 5),
            (1.0, 6),
            (1.0, 7),
        ],
    );
    generate_icosahedron_files(
        "output/",
        vec![
            (1.0, 0),
            (1.0, 1),
            (1.0, 2),
            (1.0, 3),
            (1.0, 4),
            (1.0, 5),
            (1.0, 6),
            (1.0, 7),
        ],
    );
}
