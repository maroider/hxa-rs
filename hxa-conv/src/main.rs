use std::convert::Infallible;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use pico_args::Arguments;

fn main() {
    let mut args = Arguments::from_env();
    let file = args
        .free_from_os_str::<_, Infallible>(|s| Ok(PathBuf::from(s)))
        .unwrap();
    let target_format: Format = args.value_from_str("--to").unwrap();
    assert!(file.exists(), "{} does not exist", file.display());
    convert_to(&file, target_format);
}

fn convert_to(source: &Path, target_format: Format) {
    let source_format = detect_source_format(&source);

    match source_format {
        Format::Hxa => match target_format {
            f @ Format::Hxa => identical_format(source, f),
            Format::Obj => convert_hxa_to_obj(source),
        },
        Format::Obj => match target_format {
            Format::Hxa => convert_obj_to_hxa(source),
            f @ Format::Obj => identical_format(source, f),
        },
    }
}

fn identical_format(source: &Path, format: Format) {
    println!("{} is already in the {} format", source.display(), format);
}

fn detect_source_format(file: &Path) -> Format {
    Format::from_str(file.extension().unwrap().to_str().unwrap()).unwrap()
}

#[derive(Clone, Copy, Debug)]
enum Format {
    Hxa,
    Obj,
}

impl FromStr for Format {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hxa" => Ok(Format::Hxa),
            "obj" => Ok(Format::Obj),
            _ => Err("Invalid format"),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hxa => write!(f, "HxA"),
            Self::Obj => write!(f, "Wavefront OBJ"),
        }
    }
}

fn convert_hxa_to_obj(source: &Path) {
    use hxa::{Hxa, LayerData, NodeContent, NodeType};
    use obj::{Obj, ObjData};

    let data = fs::read(source).unwrap();
    let hxa = Hxa::new(&data).unwrap();
    let mut obj = Obj {
        data: ObjData::default(),
        path: PathBuf::new(),
    };

    println!("Warning: The {0} to {1} conversion is currently incomplete, and may omit important parts of your {0} file", Format::Hxa, Format::Obj);

    for node in hxa.nodes {
        println!(r#"Encountered node of type "{:?}""#, node.type_);
        println!("Ignoring {} pieces of metadata", node.metadata.len());

        match node.content {
            Some(NodeContent::Geometry(geometry)) => {
                let vertex_stack = geometry.vertex_stack.layers;
                let corner_stack = geometry.corner_stack.layers;
                let vertices = &vertex_stack[hxa::HC_BASE_VERTEX_LAYER_ID];
                assert_eq!(vertices.name, hxa::HC_BASE_VERTEX_LAYER_NAME);
                assert_eq!(
                    vertices.component_count,
                    hxa::HC_BASE_VERTEX_LAYER_COMPONENTS
                );
                let indices = &corner_stack[hxa::HC_BASE_CORNER_LAYER_ID];
                assert_eq!(indices.name, hxa::HC_BASE_CORNER_LAYER_NAME);
                assert_eq!(
                    indices.component_count,
                    hxa::HC_BASE_CORNER_LAYER_COMPONENTS
                );
                match &vertices.data {
                    LayerData::Double(slice) => {
                        for vertex in slice.chunks_exact(vertices.component_count as usize) {
                            //
                        }
                    }
                    LayerData::Float(vertices) => {
                        //
                    }
                    _ => println!("Non-floating-point vertex data is currently unsupported"),
                }
                todo!()
            }
            Some(NodeContent::Image(_)) => {
                println!(
                    "Images cannot be directly included in a {} file",
                    Format::Obj
                )
            }
            None => match node.type_ {
                t @ NodeType::Geometry | t @ NodeType::Image => {
                    println!(
                        r#"Nodes of type "{:?}" should have content, but this one doesn't"#,
                        t
                    )
                }
                NodeType::Meta => {}
            },
        }
    }
    todo!()
}

fn convert_obj_to_hxa(_source: &Path) {
    todo!()
}
