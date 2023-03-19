use clap::Parser;
use serde::Serialize;
use std::process;

/// Fast and simple data serialization format converter
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    source: String,
    target: String,

    /// Optimize size of json and toml output by removing whitespace
    #[arg(long, short, action)]
    optimize_size: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Value {
    Json(serde_json::Value),
    Toml(toml::Value),
    Yaml(serde_yaml::Value),
}

fn get_source_value(source_ext: &str, source_content: String) -> Value {
    match source_ext {
        "json" => Value::Json(serde_json::from_str(source_content.as_str()).expect("Invalid JSON")),
        "yaml" => Value::Yaml(serde_yaml::from_str(source_content.as_str()).expect("Invalid YAML")),
        "toml" => Value::Toml(toml::from_str(source_content.as_str()).expect("Invalid TOML")),
        _ => {
            eprintln!("File type not supported: {}", source_ext);
            process::exit(1);
        }
    }
}

fn get_target_value(target_ext: &str, source_value: Value, opt_size: bool) -> String {
    match target_ext {
        "json" => {
            if opt_size {
                serde_json::to_string(&source_value).expect("Could not serialize to JSON")
            } else {
                serde_json::to_string_pretty(&source_value).expect("Could not serialize to JSON")
            }
        }
        "yaml" => serde_yaml::to_string(&source_value).expect("Could not serialize to YAML"),
        "toml" => {
            if opt_size {
                toml::to_string(&source_value).expect(
                    "Could not serialize to TOML, probably because can't stringify arrays only objects",
                )
            } else {
                toml::to_string_pretty(&source_value).expect(
                    "Could not serialize to TOML, probably because can't stringify arrays only objects",
                )
            }
        }
        _ => {
            eprintln!("File type not supported: {}", target_ext);
            process::exit(1);
        }
    }
}

fn main() {
    let args = Cli::parse();
    let source = args.source.split('.').collect::<Vec<&str>>();
    let target = args.target.split('.').collect::<Vec<&str>>();

    if source.len() < 2 || target.len() < 2 {
        eprintln!("The file needs to have an extension!");
        process::exit(1);
    }

    let source_name = source[0];
    let source_ext = source[1];

    let target_name = target[0];
    let target_ext = target[1];

    let source_content =
        std::fs::read_to_string(source_name.to_string() + "." + source_ext).unwrap();

    let source_value = get_source_value(source_ext, source_content);

    let target_value = get_target_value(target_ext, source_value, args.optimize_size);

    std::fs::write(target_name.to_string() + "." + target_ext, target_value)
        .expect("Unable to write file");
}
