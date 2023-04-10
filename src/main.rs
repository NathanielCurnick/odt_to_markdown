use std::{env, fs::File, io::Write, os::unix::prelude::FileExt, path::PathBuf};

use odt::read_odt;

mod odt;

fn main() {
    if let Some(path) = env::args().nth(1) {
        let path = PathBuf::from(path);
        let markdown = match read_odt(path.clone()) {
            Ok(x) => x,
            Err(y) => panic!("Could not convert to markdown because {}", y),
        };

        let parent = path.parent().unwrap();

        let path_no_extension = path.with_extension("");
        let file_name = path_no_extension.file_name().unwrap().to_str().unwrap();

        let mut markdown_file_name = String::new();
        markdown_file_name.push_str(file_name);
        markdown_file_name.push_str(".md");

        let markdown_path = parent.join(markdown_file_name);

        let mut file = File::create(markdown_path).unwrap();
        file.write_all(&markdown.as_bytes()).unwrap();
    } else {
        println!("USAGE: odt_to_markdown [PATH-TO-ODT-FILE]")
    }
}
