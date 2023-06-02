use clap::Parser;
use quickxml_to_serde::{Config, NullValue};
use serde::{Deserialize, Serialize};
use std::io::IsTerminal;
use std::io::{self, BufWriter, Read, Write};
use std::{ffi::OsStr, fs::File, path::Path, process};

/// Fast and simple data serialization format converter
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    source: Option<String>,
    target: Option<String>,

    /// Optimize size of json, xml and toml output by removing whitespace
    #[arg(long, short)]
    optimize_size: bool,

    /// Root element name for xml output
    #[arg(long, short)]
    root_element: Option<String>,

    /// Explicit format; If not specified, it will be inferred from the file extension
    #[arg(long, short)]
    from: Option<Format>,

    /// Explicit format; If not specified, it will be inferred from the file extension
    #[arg(long, short)]
    to: Option<Format>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Value {
    Json(serde_json::Value),
    Toml(toml::Value),
    Yaml(serde_yaml::Value),
    Xml(serde_json::Value),
}

enum WriterType {
    Stdout(BufWriter<io::Stdout>),
    File(BufWriter<File>),
}

impl Write for WriterType {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            WriterType::Stdout(writer) => writer.write(buf),
            WriterType::File(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            WriterType::Stdout(writer) => writer.flush(),
            WriterType::File(writer) => writer.flush(),
        }
    }
}

#[derive(clap::ValueEnum, Clone)]
enum Format {
    Json,
    Yaml,
    Toml,
    Xml,
}

impl std::string::ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Toml => "toml",
            Format::Xml => "xml",
        }
        .to_string()
    }
}

fn get_source_value(source_ext: &str, source_content: &str, from: Option<Format>) -> Value {
    let format = match from {
        Some(format) => format.to_string(),
        None => source_ext.to_string(),
    };
    match format.as_str() {
        "json" => Value::Json(serde_json::from_str(source_content).unwrap_or_else(|_| {
            eprintln!("Invalid JSON");
            process::exit(1);
        })),
        "yaml" | "yml" => Value::Yaml(serde_yaml::from_str(source_content).unwrap_or_else(|_| {
            eprintln!("Invalid YAML");
            process::exit(1);
        })),
        "toml" => Value::Toml(toml::from_str(source_content).unwrap_or_else(|_| {
            eprintln!("Invalid TOML");
            process::exit(1);
        })),
        "xml" => Value::Xml(
            quickxml_to_serde::xml_str_to_json(
                source_content,
                &Config::new_with_custom_values(false, "", "text", NullValue::EmptyObject),
            )
            .unwrap_or_else(|_| {
                eprintln!("Invalid XML");
                process::exit(1);
            }),
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
    root_tag: Option<&str>,
    writer: WriterType,
    in_terminal: bool,
) -> Option<String> {
    match target_ext {
        "json" => {
            if opt_size {
                if in_terminal {
                    serde_json::to_writer(writer, &source_value)
                        .expect("Could not serialize to JSON");
                    return None;
                } else {
                    return Some(
                        serde_json::to_string(&source_value).expect("Could not serialize to JSON"),
                    );
                }
            }
            if in_terminal {
                serde_json::to_writer_pretty(writer, &source_value)
                    .expect("Could not serialize to JSON");
                None
            } else {
                Some(
                    serde_json::to_string_pretty(&source_value)
                        .expect("Could not serialize to JSON"),
                )
            }
        }
        "yaml" | "yml" => {
            if in_terminal {
                serde_yaml::to_writer(writer, &source_value).expect("Could not serialize to YAML");
                None
            } else {
                Some(serde_yaml::to_string(&source_value).expect("Could not serialize to YAML"))
            }
        }
        "toml" => {
            if opt_size {
                return Some(
                    toml::to_string(&source_value)
                        .expect("Could not serialize to TOML, probably because of null values"),
                );
            }
            Some(
                toml::to_string_pretty(&source_value)
                    .expect("Could not serialize to TOML, probably because of null values"),
            )
        }
        "xml" => {
            let mut buffer = String::new();

            let mut ser = quick_xml::se::Serializer::with_root(
                &mut buffer,
                if let Some(root_tag) = root_tag {
                    Some(root_tag)
                } else {
                    Some("")
                },
            )
            .unwrap();
            if !opt_size {
                quick_xml::se::Serializer::indent(&mut ser, ' ', 4);
            }

            source_value
                .serialize(ser)
                .expect("Could not serialize to XML");

            if let None = root_tag {
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

fn read_stdin_pipe() -> String {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    buffer
}

fn write_stdout(content: String) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(content.as_bytes()).unwrap();
}

fn main() {
    let args = Cli::parse();
    let in_terminal = io::stdout().is_terminal(); // true if stdout is a terminal (not a file or pipe)

    let (target, source_value) = if io::stdin().is_terminal() {
        // ^ If reading from files

        if let (None, None) = (&args.source, &args.target) {
            eprintln!("You must specify a source and a target file!");
            process::exit(1);
        }
        let source = &args.source.unwrap();
        let source = Path::new(source);
        let source_content = std::fs::read_to_string(source).unwrap();

        let source_value = get_source_value(
            source.extension().and_then(OsStr::to_str).unwrap(),
            source_content.as_str(),
            args.from,
        );

        if in_terminal {
            let target = if let Some(ref target) = args.target {
                Path::new(target)
            } else {
                eprintln!("You must specify a target file!");
                process::exit(1);
            };
            (Some(target.to_owned()), source_value)
        } else {
            (None, source_value)
        }
    } else {
        // ^ If reading from stdin

        let source_content = read_stdin_pipe();
        let source_ext = args.from;

        let source_ext = if let Some(source_ext) = source_ext {
            source_ext
        } else {
            eprintln!("You must specify the source format when reading from stdin");
            process::exit(1);
        };

        let source_value = get_source_value(
            source_ext.to_string().as_str(),
            source_content.as_str(),
            None,
        );
        if in_terminal {
            let target = if let Some(ref source) = args.source {
                source
            } else {
                eprintln!("You must specify a target file!");
                process::exit(1);
            };

            // It's source because when executing from stdin you only specify the "target" but in the CLI parser it is taking the place of the source parameter
            let target = Path::new(target);

            (Some(target.to_owned()), source_value)
        } else {
            (None, source_value)
        }
    };

    let file = if in_terminal {
        WriterType::File(BufWriter::new(
            File::create(&target.as_ref().unwrap()).unwrap(),
        ))
    } else {
        WriterType::Stdout(BufWriter::new(io::stdout()))
    };

    let ext = if let Some(ref ext) = args.to {
        ext.to_string()
    } else {
        target
            .as_ref()
            .unwrap_or_else(|| {
                eprintln!("Unexpected error, probably you forgot to specify the --to tag");
                process::exit(1);
            })
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_else(|| {
                eprintln!("Target file has no extension");
                process::exit(1);
            })
            .to_string()
    };
    let ext = ext.as_str();

    let target_value = get_target_value(
        ext,
        source_value,
        args.optimize_size,
        args.root_element.as_deref(),
        file,
        in_terminal,
    );

    if let Some(target_value) = target_value {
        if in_terminal {
            // is stdout is a terminal (not a file or pipe)
            // if target_value is a string write it to file
            std::fs::write(target.unwrap(), target_value).expect("Unable to write file");
        } else {
            // if target_value is a string write it to stdout
            write_stdout(target_value);
        }
    }
}
