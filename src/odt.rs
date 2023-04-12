use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace};
use zip::ZipArchive;

#[derive(Clone, Copy, Debug)]
enum Style {
    Normal,
    Bold,
    Italics,
}

#[derive(Debug)]
struct HashAdder {
    name: Option<String>,
    style: Option<Style>,
}

#[derive(Debug)]
enum Tokens {
    NewLine,
    Text(String),
    StartStyle(Style),
    EndStyle,
    Heading(usize),
    Quotation,
}

impl HashAdder {
    fn default() -> Self {
        return Self {
            name: None,
            style: None,
        };
    }

    fn add_name(&mut self, name: String) {
        if self.name.is_some() {
            panic!(
                "Trying to add name '{}' to HashAdder when name '{}' already present",
                name,
                self.name.clone().unwrap()
            );
        }

        self.name = Some(name);
    }

    fn add_style(&mut self, style: Style) {
        if self.style.is_some() {
            panic!("Trying to add style to HashAdder when style already present");
        }

        self.style = Some(style);
    }

    fn reset(&mut self) {
        // Should I panic in here is name or style are None?
        self.name = None;
        self.style = None;
    }

    fn add_to_hash(&mut self, hash: &mut HashMap<String, Style>) {
        if self.style.is_none() {
            self.style = Some(Style::Normal);
        }
        if self.name.is_none() {
            panic!("Trying to add to HashMap when HashAdder is not ready");
        }

        hash.insert(self.name.clone().unwrap(), self.style.unwrap());
        self.reset();
    }
}

pub fn read_odt(path: PathBuf) -> Result<String, String> {
    let file = match File::open(path) {
        Ok(x) => x,
        Err(y) => return Err(format!("Could not open file because due to {}", y)),
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(x) => x,
        Err(y) => return Err(format!("Could not open archive due to {}", y)),
    };

    let mut xml = String::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i).unwrap();
        if c_file.name() == "content.xml" {
            c_file.read_to_string(&mut xml).unwrap();
            break;
        }
    }

    let tokens = handle_xml(&xml);

    let markdown = convert(&tokens);

    return Ok(markdown);
}

fn handle_xml(xml: &String) -> Vec<Tokens> {
    let mut hash: HashMap<String, Style> = HashMap::new();
    let mut hashadder = HashAdder::default();

    let parser = xml::reader::EventReader::from_str(xml);

    let mut tokens: Vec<Tokens> = vec![];

    for event in parser {
        match event.unwrap() {
            xml::reader::XmlEvent::StartDocument { .. } => {}
            xml::reader::XmlEvent::EndDocument => {}
            xml::reader::XmlEvent::ProcessingInstruction { .. } => {}
            xml::reader::XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => handle_start_element(&mut tokens, &mut hash, &mut hashadder, &name, &attributes),
            xml::reader::XmlEvent::EndElement { name } => handle_end_element(&mut tokens, &name),
            xml::reader::XmlEvent::CData(_) => todo!(),
            xml::reader::XmlEvent::Comment(_) => todo!(),
            xml::reader::XmlEvent::Characters(chars) => {
                tokens.push(Tokens::Text(chars));
            }
            xml::reader::XmlEvent::Whitespace(_) => todo!(),
        }
    }

    return tokens;
}

fn handle_end_element(tokens: &mut Vec<Tokens>, name: &OwnedName) {
    let name = name.to_string();

    if name.contains("text:p") {
        tokens.push(Tokens::NewLine);
    } else if name.contains("text:span") {
        tokens.push(Tokens::EndStyle);
    } else if name.contains("text:h") {
        tokens.push(Tokens::NewLine);
    }
}

fn handle_start_element(
    tokens: &mut Vec<Tokens>,
    hash: &mut HashMap<String, Style>,
    hashadder: &mut HashAdder,
    name: &OwnedName,
    attributes: &Vec<OwnedAttribute>,
) {
    let name = name.to_string();

    if name.contains("style:style") {
        for attribute in attributes {
            if attribute.name.local_name == "name" {
                hashadder.add_name(attribute.value.clone());
            }
        }
    } else if name.contains("style:text-properties") {
        for attribute in attributes {
            if attribute.name.local_name == "font-style" && attribute.value == "italic" {
                hashadder.add_style(Style::Italics);
                break;
            } else if attribute.name.local_name == "font-weight" && attribute.value == "bold" {
                hashadder.add_style(Style::Bold);
                break;
            } else {
                // hashadder.add_style(Style::Normal);
                // break;
            }
        }
        hashadder.add_to_hash(hash);
    } else if name.contains("text:p") {
        for attribute in attributes {
            if attribute.value == "Quotations" {
                tokens.push(Tokens::Quotation);
            }
            match hash.get(&attribute.value) {
                Some(x) => tokens.push(Tokens::StartStyle(x.clone())),
                None => {}
            };
        }
    } else if name.contains("text:span") {
        for attribute in attributes {
            match hash.get(&attribute.value) {
                Some(x) => tokens.push(Tokens::StartStyle(x.clone())),
                None => {}
            };
        }
    } else if name.contains("text:h") {
        for attribute in attributes {
            if attribute.name.local_name == "outline-level" {
                let heading_level = attribute.value.parse::<usize>().unwrap();
                tokens.push(Tokens::Heading(heading_level));
            }
        }
    }
}

fn convert(tokens: &Vec<Tokens>) -> String {
    let mut markdown = String::new();

    let mut started_style: Option<Style> = None;

    for token in tokens {
        match token {
            Tokens::NewLine => {
                if started_style.is_some() {
                    match started_style.unwrap() {
                        Style::Normal => {}
                        Style::Bold => markdown.push_str("**"),
                        Style::Italics => markdown.push_str("*"),
                    }
                }
                markdown.push_str("\n\n");
                started_style = None;
            }
            Tokens::Text(x) => markdown.push_str(x),
            Tokens::StartStyle(style) => {
                if started_style.is_some() {
                    match started_style.unwrap() {
                        Style::Normal => {}
                        Style::Bold => markdown.push_str("**"),
                        Style::Italics => markdown.push_str("*"),
                    }
                }

                match style {
                    Style::Normal => {
                        started_style = Some(Style::Normal);
                    }
                    Style::Bold => {
                        markdown.push_str("**");
                        started_style = Some(Style::Bold);
                    }
                    Style::Italics => {
                        markdown.push_str("*");
                        started_style = Some(Style::Italics);
                    }
                }
            }
            Tokens::EndStyle => {
                if started_style.is_none() {
                    panic!("Ending style before starting!");
                }

                match started_style.unwrap() {
                    Style::Normal => {}
                    Style::Bold => markdown.push_str("**"),
                    Style::Italics => markdown.push_str("*"),
                };

                started_style = None;
            }
            Tokens::Heading(x) => {
                for i in 0..*x {
                    markdown.push_str("#")
                }
                markdown.push_str(" ");
            }
            Tokens::Quotation => markdown.push_str(">"),
        }
    }

    return markdown;
}
