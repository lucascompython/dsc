use clap::Parser;
use quickxml_to_serde::{Config, NullValue};
use serde::Serialize;
use std::{ffi::OsStr, fs::File, io::BufWriter, path::Path, process};

/// Fast and simple data serialization format converter
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    source: String,
    target: String,

    /// Optimize size of json, xml and toml output by removing whitespace
    #[arg(long, short, action)]
    optimize_size: bool,

    /// Root element name for xml output
    #[arg(long, short, default_value = "")]
    root_element: String,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Value {
    Json(serde_json::Value),
    Toml(toml::Value),
    Yaml(serde_yaml::Value),
    Xml(serde_json::Value),
}

fn get_source_value(source_ext: &str, source_content: &str) -> Value {
    match source_ext {
        "json" => Value::Json(serde_json::from_str(source_content).expect("Invalid JSON")),
        "yaml" | "yml" => Value::Yaml(serde_yaml::from_str(source_content).expect("Invalid YAML")),
        "toml" => Value::Toml(toml::from_str(source_content).expect("Invalid TOML")),
        "xml" => Value::Xml(
            quickxml_to_serde::xml_str_to_json(
                source_content,
                &Config::new_with_custom_values(false, "", "text", NullValue::EmptyObject),
            )
            .expect("Invalid XML"),
        ),
        _ => {
            eprintln!("File type not supported: {}", source_ext);
            process::exit(1);
        }
    }
}

fn get_target_value(
    target_ext: &str,
    source_value: Value,
    opt_size: bool,
    root_tag: String,
    writer: BufWriter<File>,
) -> Option<String> {
    match target_ext {
        "json" => {
            if opt_size {
                serde_json::to_writer(writer, &source_value).expect("Could not serialize to JSON");
                None
            } else {
                serde_json::to_writer_pretty(writer, &source_value)
                    .expect("Could not serialize to JSON");
                None
            }
        }
        "yaml" | "yml" => {
            serde_yaml::to_writer(writer, &source_value).expect("Could not serialize to YAML");
            None
        }
        "toml" => {
            if opt_size {
                Some(toml::to_string(&source_value).expect(
                    "Could not serialize to TOML, probably because can't stringify arrays only objects",
                ))
            } else {
                Some(toml::to_string_pretty(&source_value).expect(
                    "Could not serialize to TOML, probably because can't stringify arrays only objects",
                ))
            }
        }
        "xml" => {
            let mut buffer = String::new();
            let mut ser =
                quick_xml::se::Serializer::with_root(&mut buffer, Some(&root_tag)).unwrap();
            if !opt_size {
                quick_xml::se::Serializer::indent(&mut ser, ' ', 4);
            }

            source_value
                .serialize(ser)
                .expect("Could not serialize to XML");

            if root_tag.is_empty() {
                buffer = buffer.replace("<>", "");
                buffer = buffer.replace("</>", "");
            }

            Some(buffer)
        }

        _ => {
            eprintln!("File type not supported: {}", target_ext);
            process::exit(1);
        }
    }
}

fn main() {
    let args = Cli::parse();

    let source = Path::new(&args.source);
    let target = Path::new(&args.target);

    let source_content = std::fs::read_to_string(source).unwrap();

    let source_value = get_source_value(
        source.extension().and_then(OsStr::to_str).unwrap(),
        source_content.as_str(),
    );

    let file = BufWriter::new(File::create(target).unwrap());

    let target_value = get_target_value(
        target.extension().and_then(OsStr::to_str).unwrap(),
        source_value,
        args.optimize_size,
        args.root_element,
        file,
    );

    if let Some(target_value) = target_value {
        std::fs::write(target, target_value).expect("Unable to write file");
    }
}
