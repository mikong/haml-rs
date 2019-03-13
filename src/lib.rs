/* mod ast;
mod common;
mod generator;
mod parser;
mod scanner;
mod values;

use ast::ToAst;
use parser::Parser;
use scanner::Scanner;

/// Determines what type of output to generate
/// *  XHtml will generate valid xhtml
/// *  Html5 will generate valid html based on HTML5 spec
/// *  Html4 will generate valid html based on HTML4 spec
#[derive(Clone, Debug)]
pub enum HtmlFormat {
    XHtml(),
    Html5(),
    Html4(),
}

/// Converts the Haml that is contained in a reference string
/// into an owned string.
///
/// # Arguments
/// *  `haml` - A string slice that contains the Haml to be converted to HTML
/// *  `format` - A HtmlFormat enum that determines what kind of Html output
/// will be generated
///
/// ## Examples
///
/// ### Converting Haml to Html5
/// ```rust
/// use haml;
/// let html: String = haml::to_html("%span", HtmlFormat::Html5());
/// ```
///
/// ### Converting Haml to Html4
/// ```rust
/// use haml;
/// let html: String = haml::to_html("%span", HtmlFormat::Html4());
/// ```
///
/// ### Converting Haml to xhtml
/// ```rust
/// use haml;
/// let html: String = haml::to_html("%span", HtmlFormat::XHtml());
/// ```
pub fn to_html(haml: &str, format: HtmlFormat) -> String {
    let mut scanner = Scanner::new(haml);
    let tokens = scanner.get_tokens();
    let mut parser = Parser::new(tokens);
    let parsed_values = parser.parse();
    generator::to_html(&parsed_values, format)
}

pub fn parse(haml: &str, format: HtmlFormat) -> String {
    "".to_string()
}

/// Converts the Haml into an abstract syntax tree
///
/// # Arguments
/// *  `haml` - A string slice that contains the Haml to be parsed
/// output is generated
///
/// ## Examples
///
/// ```rust
/// use haml;
/// let ast = haml::to_ast("%span");
/// ```
pub fn to_ast(haml: &str) -> String {
    let mut scanner = Scanner::new(haml);
    let tokens = scanner.get_tokens();
    let mut parser = Parser::new(tokens);
    parser.parse().to_ast()
}
*/
// extern crate regex;
// mod pipes;
mod lex;
