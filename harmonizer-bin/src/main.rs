use std::{fs::read_to_string, path::PathBuf};

use harmonizer::compose::{compose, ServiceDefinition};

fn main() {
    let composed = compose(
        std::env::args()
            .skip(1)
            .map(|file| {
                let src = read_to_string(&file).expect("reading source file");
                ServiceDefinition::new(
                    PathBuf::from(&file)
                        .file_stem()
                        .expect("path must point to a schema file")
                        .to_str()
                        .expect("os string decoding"),
                    &file,
                    src,
                )
            })
            .collect(),
    );

    match composed {
        Ok(schema) => {
            println!("{}", schema);
        }
        Err(errors) => {
            eprintln!(
                "{count} {errors} occurred during composition:",
                count = errors.len(),
                errors = if errors.len() == 1 { "error" } else { "errors" }
            );

            for (index, err) in errors.iter().enumerate() {
                if let Some(ref msg) = err.message {
                    eprintln!(
                        "  {index}. {code}: {message}",
                        index = index + 1,
                        code = err.code(),
                        message = msg
                    );
                } else {
                    eprintln!("  {index}. {code}", index = index + 1, code = err.code())
                }
            }
        }
    }
}
