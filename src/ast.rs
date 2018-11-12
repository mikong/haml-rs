use super::HtmlFormat;
use common;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Attributes {
    attributes: HashMap<String, Vec<String>>,
}

impl Attributes {
    pub fn new() -> Attributes {
        Attributes {
            attributes: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: String, value: String) {
        if self.attributes.get(&key) == None {
            self.attributes.insert(key.clone(), vec![]);
        }
        if let Some(attrs) = self.attributes.get_mut(&key) {
            (*attrs).push(value);
        }
    }

    pub fn raw(&self) -> &HashMap<String, Vec<String>> {
        &self.attributes
    }
}

impl ToHtml for Attributes {
    fn to_html(&self, format: HtmlFormat) -> String {
        let mut html_builder = String::new();
        for key in self.attributes.keys() {
            let values = self.attributes.get(key).unwrap().join(" ");
            html_builder.push_str(&format!(" {}=\"{}\"", key, values));
        }
        html_builder
    }
}

pub trait ToHtml {
    fn to_html(&self, format: HtmlFormat) -> String;
}

pub trait ToAst {
    fn to_ast(&self) -> String;
}

#[derive(Clone, Debug)]
pub enum Html {
    Comment(String),
    Text(String),
    Doctype(String),
    Element(HtmlElement),
    SilentComment(String),
    Css(CssElement),
}
impl ToAst for Html {
    fn to_ast(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct CssElement {
    pub text: String,
}

impl CssElement {
    pub fn new(text: String) -> CssElement {
        CssElement { text }
    }
}

#[derive(Clone, Debug)]
pub struct HtmlElement {
    pub tag: String,
    pub attributes: Attributes,
    pub body: String,
}

impl HtmlElement {
    pub fn new(tag: String) -> HtmlElement {
        HtmlElement {
            tag,
            attributes: Attributes::new(),
            body: String::new(),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub fn add_attribute(&mut self, key: String, value: String) {
        let clean_value = value.replace("'", " ").trim().to_string();
        self.attributes.add(key, clean_value);
    }
}

#[derive(Debug, Clone)]
pub struct Arena {
    nodes: Vec<Node>,
    levels: HashMap<u32, usize>,
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            nodes: vec![],
            levels: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, child_id: usize, parent_id: usize) {
        self.nodes[parent_id].children.push(child_id);
        self.nodes[child_id].parent = parent_id;
    }

    pub fn add_sibling(&mut self, current_id: usize, sibling_id: usize) {
        self.nodes[current_id].next_sibling = Some(sibling_id);
        self.nodes[sibling_id].parent = self.parent(current_id);
    }

    pub fn parent(&self, id: usize) -> usize {
        if self.nodes.len() > 0 {
            self.nodes[id].parent
        } else {
            0
        }
    }

    pub fn new_node(&mut self, data: Html, indentation: u32) -> usize {
        let next_index = self.nodes.len();
        self.nodes.push(Node {
            parent: 0,
            children: vec![],
            next_sibling: None,
            data,
            indentation,
        });
        self.levels.insert(indentation, next_index);

        next_index
    }

    pub fn node_at(&self, id: usize) -> &Node {
        &self.nodes[id]
    }

    fn node_to_ast(&self, id: usize, indent: &str) -> String {
        let mut ast_builder = String::new();
        let node = self.node_at(id);
        ast_builder.push_str(&format!("{:?}", node.data));
        for child in node.children() {
            ast_builder.push_str(&format!(
                "\n{}{}",
                indent,
                self.node_to_ast(*child, &format!("{}\t", indent))
            ));
        }
        ast_builder
    }
}

impl ToAst for Arena {
    fn to_ast(&self) -> String {
        if self.nodes.len() > 0 {
            self.node_to_ast(0, "")
        } else {
            "".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    parent: usize,
    next_sibling: Option<usize>,
    children: Vec<usize>,
    pub data: Html,
    indentation: u32,
}

impl Node {
    pub fn next_sibling(&self) -> Option<usize> {
        self.next_sibling
    }

    pub fn children(&self) -> &Vec<usize> {
        &self.children
    }
}
