use clap::Parser;
use serde::Serialize;
use std::process;

#[derive(Parser)]
struct Cli {
    source: String,
    target: String,
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
        "json" => Value::Json(serde_json::from_str(source_content.as_str()).unwrap()),
        "yaml" => Value::Yaml(serde_yaml::from_str(source_content.as_str()).unwrap()),
        "toml" => Value::Toml(toml::from_str(source_content.as_str()).unwrap()),
        _ => {
            eprintln!("File type not supported: {}", source_ext);
            process::exit(1);
        }
    }
}

fn get_target_value(target_ext: &str, source_value: Value) -> String {
    match target_ext {
        "json" => serde_json::to_string_pretty(&source_value).unwrap(),
        "yaml" => serde_yaml::to_string(&source_value).unwrap(),
        "toml" => toml::to_string_pretty(&source_value).unwrap(),
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

    let target_value = get_target_value(target_ext, source_value);

    std::fs::write(target_name.to_string() + "." + target_ext, target_value)
        .expect("Unable to write file");
}
