use crate::builtins::BuiltIn;

#[derive(Debug, PartialEq, Clone)]
/// Previously known as `Sexpr`
pub enum LispValue {
    List(Vec<LispValue>),
    Quote(Box<LispValue>),
    QuasiQuotedList(Vec<LispValue>),
    Symbol(String),
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Function {
        parameters: Vec<String>,
        body: Vec<LispValue>,
    },
    Macro {
        parameters: Vec<String>,
        body: Box<LispValue>,
    },
    BuiltIn(BuiltIn),
    CommaUnquote(Box<LispValue>),
    Nil,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SrcSexpr {
    Bool(bool), // true, false
    Int(i64), // 1
    Float(f64), // 1.0
    String(String), // "foo"
    Symbol(String), // +, -, *, /, foo
    List(Vec<SrcSexpr>), // (+ 2 3)
    Quote(Box<SrcSexpr>), // '(+ 2 3), 'foo
}

impl SrcSexpr {
    pub fn to_sexpr(&self) -> LispValue {
        match self {
            SrcSexpr::List(sexprs) => LispValue::List(sexprs.iter().map(|t| t.to_sexpr()).collect()),
            SrcSexpr::Symbol(s) => LispValue::Symbol(s.clone()),
            SrcSexpr::String(s) => LispValue::String(s.clone()),
            SrcSexpr::Bool(b) => LispValue::Bool(*b),
            SrcSexpr::Int(i) => LispValue::Int(*i),
            SrcSexpr::Float(f) => LispValue::Float(*f),
            SrcSexpr::Quote(sexpr) => LispValue::Quote(Box::new(sexpr.to_sexpr())),
        }
    }
}
