use std::{
    env,
    fs::File,
    io::Write,
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
};

use odt::read_odt;

mod odt;

#[cfg(test)]
mod tests;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    if args.contains(&"-h".to_string()) {
        print_usage();
        return;
    }

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

        let markdown_path = parent.join(markdown_file_name.clone());

        let mut file = File::create(markdown_path.clone()).unwrap();
        file.write_all(&markdown.as_bytes()).unwrap();
        println!("Written output to {}", markdown_path.to_string_lossy());
    } else {
        print_usage();
    }
}

fn print_usage() {
    println!("USAGE: odt_to_md <options> [PATH-TO-ODT-FILE]");
    println!("Options:");
    println!("\t-h: This help");
}
