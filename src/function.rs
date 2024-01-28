// use crate::sturctural_parser::StructuredNode;

use crate::sturctural_parser::StructuredNode;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub params: Vec<String>,
    pub body: StructuredNode,
}

// const MAP = Function {
//     params: vec!["f".to_string(), "list".to_string()],
//     ///
//     /// (fn (f list)
//     ///     (if (empty? list)
//     ///         list
//     ///         (cons (f (car list)) (map f (cdr list)))))
//     ///
//     ///
//     body: lex
// }

// pub const PRELUDE: [Function; 1] = [MAP]
