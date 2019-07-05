#[macro_use]
extern crate clap;
extern crate byteorder;
extern crate icosahedron;

use std::fs::{metadata, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};
use icosahedron::Polyhedron;

fn write_to_binary_file(polyhedron: Polyhedron, path: &Path) {
    let bin_file = File::create(path).expect("Can't create file");
    let mut writer = BufWriter::new(bin_file);
    let write_error_message = "Error encountered while writing to binary file";
    writer
        .write_u32::<LittleEndian>(polyhedron.positions.len() as u32)
        .expect(write_error_message);
    writer
        .write_u32::<LittleEndian>(polyhedron.cells.len() as u32)
        .expect(write_error_message);
    for position in polyhedron.positions.iter() {
        writer
            .write_f32::<LittleEndian>(position.0.x)
            .expect(write_error_message);
        writer
            .write_f32::<LittleEndian>(position.0.y)
            .expect(write_error_message);
        writer
            .write_f32::<LittleEndian>(position.0.z)
            .expect(write_error_message);
    }
    for normal in polyhedron.normals.iter() {
        writer
            .write_f32::<LittleEndian>(normal.0.x)
            .expect(write_error_message);
        writer
            .write_f32::<LittleEndian>(normal.0.y)
            .expect(write_error_message);
        writer
            .write_f32::<LittleEndian>(normal.0.z)
            .expect(write_error_message);
    }
    for color in polyhedron.colors.iter() {
        writer
            .write_f32::<LittleEndian>(color.0.x)
            .expect(write_error_message);
        writer
            .write_f32::<LittleEndian>(color.0.y)
            .expect(write_error_message);
        writer
            .write_f32::<LittleEndian>(color.0.z)
            .expect(write_error_message);
    }
    for cell in polyhedron.cells.iter() {
        writer
            .write_u32::<LittleEndian>(cell.a as u32)
            .expect(write_error_message);
        writer
            .write_u32::<LittleEndian>(cell.b as u32)
            .expect(write_error_message);
        writer
            .write_u32::<LittleEndian>(cell.c as u32)
            .expect(write_error_message);
    }
}

fn write_to_json_file(polyhedron: Polyhedron, path: &Path) {
    let mut json_file = File::create(path).expect("Can't create file");
    let json = serde_json::to_string(&polyhedron).expect("Problem serializing");
    json_file
        .write_all(json.as_bytes())
        .expect("Can't write to file");
}

fn generate_files(
    dir: &str,
    format: Format,
    truncated: bool,
    colored: bool,
    param_list: Vec<(f32, u32)>,
) {
    let mesh_type = if truncated {
        "hexsphere"
    } else {
        "icosahedron"
    };

    for param in param_list {
        println!(
            "Generating {} with radius {} and detail {}...",
            mesh_type, param.0, param.1
        );

        let polyhedron = if truncated {
            let mut hexsphere = Polyhedron::new_truncated_isocahedron(param.0, param.1);
            hexsphere.compute_triangle_normals();
            hexsphere
        } else {
            let mut icosahedron = Polyhedron::new_isocahedron(param.0, param.1);
            icosahedron.compute_triangle_normals();
            icosahedron
        };

        let colored_polyhedron = if colored {
            let mut colored = Polyhedron::new();
            colored.unique_vertices(polyhedron);
            colored.assign_random_face_colors();
            colored
        } else {
            polyhedron
        };

        println!("triangles: {}", colored_polyhedron.cells.len());
        println!("vertices: {}", colored_polyhedron.positions.len());

        let filename = Path::new(dir).join(format!(
            "{}_r{}_d{}.{}",
            mesh_type,
            param.0,
            param.1,
            format.extension()
        ));
        match format {
            Format::Bin => write_to_binary_file(colored_polyhedron, &filename),
            Format::Json => write_to_json_file(colored_polyhedron, &filename),
        };
    }
}

arg_enum! {
    #[derive(Debug)]
    enum Format {
        Json,
        Bin,
    }
}

impl Format {
    fn extension(&self) -> String {
        match self {
            Format::Bin => "bin".to_string(),
            Format::Json => "json".to_string(),
        }
    }
}

fn main() {
    let dir_exists = |path: String| {
        let path_clone = path.clone();
        match metadata(path) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    Ok(())
                } else {
                    Err(String::from(format!(
                        "Output '{}' is not a directory",
                        &path_clone
                    )))
                }
            }
            Err(_) => Err(String::from(format!(
                "Directory '{}' doesn't exist",
                &path_clone
            ))),
        }
    };

    let matches = clap_app!(icosahedron =>
        (version: "0.1.1")
        (author: "Tyler Hallada <tyler@hallada.net>")
        (about: "Generates 3D icosahedra meshes")
        (@arg truncated: -t --truncated "Generate truncated icosahedra (hexspheres).")
        (@arg colored: -c --colored "Assigns a random color to every face \
            (increases vertices count).")
        (@arg detail: -d --detail +takes_value default_value("7")
            "Maximum detail level to generate. \
            Each level multiplies the number of triangles by 4.")
        (@arg radius: -r --radius +takes_value default_value("1.0")
            "Radius of the polyhedron,")
        (@arg format: -f --format +takes_value possible_values(&Format::variants())
            default_value("Bin")
            "Format to write the files in.")
        (@arg output: [OUTPUT] {dir_exists} default_value("output/")
            "Directory to write the output files to.")
    )
    .get_matches();

    let truncated = matches.is_present("truncated");
    let colored = matches.is_present("colored");
    let detail = value_t!(matches.value_of("detail"), u32).unwrap_or(7);
    let radius = value_t!(matches.value_of("radius"), f32).unwrap_or(1.0);
    let format = value_t!(matches.value_of("format"), Format).unwrap_or(Format::Bin);
    let output = matches.value_of("output").unwrap_or("output/");

    let param_list = |detail: u32, radius: f32| -> Vec<(f32, u32)> {
        let mut params = vec![];
        for detail in 0..(detail + 1) {
            params.push((radius, detail));
        }
        params
    };

    generate_files(
        output,
        format,
        truncated,
        colored,
        param_list(detail, radius),
    );
}
