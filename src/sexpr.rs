use crate::builtins::BuiltIn;


#[derive(Debug, PartialEq, Clone)]
pub enum Sexpr {
    List {
        quasiquote: bool,
        sexprs: Vec<Sexpr>,
    },
    // List(Vec<Sexpr>),
    Symbol(String),
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Function {
        parameters: Vec<String>,
        body: Vec<Sexpr>,
    },
    Macro {
        parameters: Vec<String>,
        body: Box<Sexpr>,
    },
    BuiltIn(BuiltIn),
    CommaUnquote(Box<Sexpr>),
    Nil,
}
