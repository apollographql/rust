use crate::commands::Command;
use crate::commands::Print;

use graphql_parser::parse_schema;
use std::fs;

impl Command for Print {
    fn run(&self) {
        let printing_headers = !self.no_headers && self.files.len() > 1;
        self.files.iter().for_each(move |file| {
            let schema = fs::read_to_string(file).expect("reading schema");
            let doc = parse_schema::<&str>(&schema).expect("parsing schema");
            if printing_headers {
                println!("# {}", file.to_str().expect("filename"));
            }
            if self.ast {
                println!("{:#?}", doc)
            } else {
                println!("{}", doc);
            }
        })
    }
}

#[test]
fn does_not_fail_with_no_files() {
    Print {
        files: vec![],
        ast: false,
        no_headers: false,
    }
    .run();
    Print {
        files: vec![],
        ast: false,
        no_headers: true,
    }
    .run();
}
