mod doctype;
pub mod element;

use crate::regex::{prolog, COMMENT_REGEX, TEXT_REGEX, sanitize};
use crate::Format;
use doctype::Doctype;
use element::{Element, ElementType};
use regex::Regex;
use std::collections::HashMap;


fn sanitize_html(line: &str) -> Option<String> {
    let r = Regex::new(&sanitize()).unwrap();
    match r.is_match(line) {
        true => {
            let caps = r.captures(line).unwrap();
            match caps.name("text") {
                Some(m) => Some(m.as_str()
                        .replace("&", "&amp;")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;")
                        .replace("'", "&apos;")
                        .replace("\"", "&quot;")
                        .to_owned()),
                None => None
            }
        }
        false => None
    }
}

fn text_from_string(line: &str) -> Option<String> {
    let r = Regex::new(TEXT_REGEX).unwrap();
    match r.captures(line) {
        Some(m) => {
            match m.name("text") {
                Some(n) => Some(n.as_str().to_owned()),
                None => None
            }
        },
        None => None
    }
}

fn comment(line: &str) -> Option<String> {
    let r = Regex::new(COMMENT_REGEX).unwrap();
    match r.is_match(line) {
        true => {
            let caps = r.captures(line).unwrap();
            if let Some(c) = caps.name("comment") {
                Some(c.as_str().trim().to_owned())
            } else {
                None
            }
        }
        false => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Haml {
    Root(),
    Element(Element),
    Text(String),
    InnerText(String),
    Comment(String),
    Prolog(String),
    Temp(String, u32, u32),
}

pub struct Parser {
    arena: Arena,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            arena: Arena::new(),
        }
    }

    pub fn parse(&mut self, haml: &str, format: &Format) -> &Arena {
        let mut previous_id = 0;
        let mut first_line = true;
        let prolog_regex = Regex::new(&prolog()).unwrap();
        for line in haml.lines() {
            // matches lines that start with &=
            if let Some(sanitized_html) = sanitize_html(line) {
                self.arena.insert(Haml::Text(sanitized_html), previous_id);
            } else if let Some(el) = Element::from_string(line) {
                let ws = el.whitespace;
                let element = Haml::Element(el);
                if !first_line {
                    let p_id = self.arena.from_whitespace(previous_id, ws);
                    previous_id = self.arena.insert(element, p_id);
                } else {
                    previous_id = self.arena.insert(element, 0);
                    first_line = false;
                }
            } else if let Some(comment) = comment(line) {
                self.arena.insert(Haml::Comment(comment), previous_id);
            } else if prolog_regex.is_match(line) {
                let caps = prolog_regex.captures(line).unwrap();
                let value = match caps.name("type") {
                    Some(m) => Some(m.as_str()),
                    _ => None,
                };
                let doctype = Doctype::new(&format, value);
                self.arena.insert(Haml::Prolog(doctype.to_html()), previous_id);
            } else if let Some(text_line) = text_from_string(line) {
                self.arena.insert(Haml::Text(text_line), previous_id);
            }
        }
        &self.arena
    }
}

#[derive(Debug)]
pub struct Arena {
    items: Vec<ArenaItem>,
}

#[derive(Debug)]
pub struct ArenaItem {
    pub value: Haml,
    pub parent: usize,
    pub children: Vec<usize>,
}

impl ArenaItem {
    pub fn new(value: Haml, parent: usize) -> ArenaItem {
        ArenaItem {
            value,
            parent,
            children: vec![],
        }
    }
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            items: vec![ArenaItem::new(Haml::Root(), 0)],
        }
    }

    pub fn insert(&mut self, haml: Haml, parent: usize) -> usize {
        self.items.push(ArenaItem::new(haml, parent));
        let idx: usize = self.items.len() - 1;
        if idx > 0 {
            self.items[parent].children.push(idx);
        }
        idx
    }

    pub fn parent(&self, i: usize) -> usize {
        self.items[i].parent
    }

    pub fn children_of(&self, i: usize) -> &Vec<usize> {
        &self.items[i].children
    }

    pub fn item(&self, i: usize) -> &ArenaItem {
        &self.items[i]
    }

    pub fn root(&self) -> &ArenaItem {
        &self.items[0]
    }

    pub fn from_whitespace(&self, start_index: usize, ws: usize) -> usize {
        let mut idx = start_index;
        let mut parent = start_index;
        loop {
            let i = &self.items[idx];
            if let Haml::Element(el) = &i.value {
                if el.whitespace < ws {
                    parent = idx;
                    break;
                }
            }
            idx = i.parent;
        }
        parent
    }

    pub fn to_html(&self) -> String {
        let mut html = String::new();
        let root = self.root();
        for child in root.children.iter() {
            html.push_str(&self.item_to_html(self.item(*child)));
        }
        html.trim().to_owned()
    }

    fn item_to_html(&self, item: &ArenaItem) -> String {
        match &item.value {
            Haml::Text(text) => format!("{}\n",text.to_owned()),
            Haml::Comment(comment) => format!("<!-- {} -->", comment.trim()),
            Haml::Element(_) => self.element_to_html(&item),
            Haml::InnerText(text) => text.to_owned(),
            Haml::Prolog(prolog) => prolog.to_owned(),
            _ => String::new(),
        }
    }

    fn element_to_html(&self, item: &ArenaItem) -> String {
        let mut html = String::new();
        if let Haml::Element(el) = &item.value {
            html.push_str(&format!("<{}", el.name().unwrap()));
            for key in el.attributes().iter() {
                if let Some(value) = el.get_attribute(key) {
                    // this needs to be separated eventuallyas this is html5 specific
                    if key.trim() == "checked" && value == "true" {
                        html.push_str(" checked");
                    } else {
                        html.push_str(&format!(" {}='{}'", key.trim(), value.to_owned()));
                    }
                }
            }
            html.push('>');
            if let Some(text) = &el.inline_text {
                html.push_str(&format!("{}", text.trim()));
            } 
            if item.children.len() > 0 {
                let mut index = 0;
                if Some("pre".to_owned()) != el.name() {
                    html.push('\n');
                }
                for c in item.children.iter() {
                    let i = self.item(*c);
                    let mut cur = self.item_to_html(i).to_owned();
                    if index + 1 == item.children.len() {
                        // let temp = 
                        cur = cur.trim_end().to_owned(); //temp;
                    }
                    html.push_str(&cur);
                    index += 1;
                }
            }
            html.push_str(&format!("</{}>\n", el.name().unwrap()));
        }
        println!("{}", html);
        html
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn parse_text() {
//         let haml = r"\= test";
//         let mut p = Parser::new();
//         let e = p.parse(haml);
//         let id = e.root().children[0];
//         let item = e.item(id);
//         match &item.value {
//             Haml::Text(ref text) => assert_eq!("= test".to_owned(), *text),
//             _ => panic!("failed"),
//         }
//     }

//     #[test]
//     fn parse_element_text() {
//         let haml = "%hi\n\\value";
//         let mut p = Parser::new();
//         let e = p.parse(haml);
//         let id = e.root().children[0];
//         let item = e.item(id);
//         if let Haml::Element(el) = &item.value {
//             let mut it = item.children.iter();
//             match it.next() {
//                 Some(child_id) => {
//                     let child = e.item(*child_id);
//                     match &child.value {
//                         Haml::Text(ref txt) => assert_eq!("value".to_owned(), *txt),
//                         _ => panic!("Failed"),
//                     }
//                 },
//                 None => panic!("Failed"),
//             }
//         }
//     }

//     #[test]
//     fn parse_element() {
//         let haml = "%hi\n  .box\n    #b\n  %span";
//         let mut p = Parser::new();
//         let e = p.parse(haml);
//         let id = e.item(0).children[0];
//         let item = e.item(id);
//         let el = match &item.value {
//             Haml::Element(el) => el,
//             _ => panic!("failed"),
//         };

//         assert_eq!(Some("%hi".to_owned()), el.name);
//         assert_eq!(ElementType::Other(), el.element_type);
//         assert_eq!(0, el.whitespace);

//         let mut it = item.children.iter();
//         let b = it.next().unwrap();
//         let bel = e.item(*b);
//         let el2 = match &bel.value {
//             Haml::Element(el) => el,
//             _ => panic!("failed")
//         };
//         assert_eq!(Some(".box".to_owned()), el2.name);
//         assert_eq!(ElementType::Div(), el2.element_type);
//         assert_eq!(2, el2.whitespace);

//         let mut it2 = bel.children.iter();
//         let c = it2.next().unwrap();
//         let cel = e.item(*c);
//         let el3 = match &cel.value {
//             Haml::Element(el) => el,
//             _ => panic!("failed")
//         };
//         assert_eq!(Some("#b".to_owned()), el3.name);
//         assert_eq!(ElementType::Div(), el3.element_type);
//         assert_eq!(4, el3.whitespace);

//         let mut d = it.next().unwrap();
//         let del = e.item(*d);
//         let el4 = match &del.value {
//             Haml::Element(el) => el,
//             _ => panic!("failed")
//         };
//         assert_eq!(Some("%span".to_owned()), el4.name);
//         assert_eq!(ElementType::Other(), el4.element_type);
//         assert_eq!(2, el4.whitespace);

//     }
// }
