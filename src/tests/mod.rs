use difference;
use std::{fs::File, io::Read, path::PathBuf};

use crate::odt::read_odt;

#[test]
fn test_sample() {
    let odt_path = PathBuf::from("samples/sample.odt");
    let generated_markdown = match read_odt(odt_path.clone()) {
        Ok(x) => x,
        Err(y) => panic!("Could not convert to markdown because {}", y),
    };

    let md_path = PathBuf::from("samples/sample.md");
    let mut md_file = File::open(md_path).unwrap();
    let mut correct_markdown = String::new();
    md_file.read_to_string(&mut correct_markdown).unwrap();

    difference::assert_diff!(&correct_markdown, &generated_markdown, " ", 0);
}
