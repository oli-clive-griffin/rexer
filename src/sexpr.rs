use crate::builtins::BuiltIn;

#[derive(Debug, PartialEq, Clone)]
/// Previously known as `Sexpr`
pub enum EvauluatorRuntimeValue {
    List(Vec<EvauluatorRuntimeValue>),
    Quote(Box<EvauluatorRuntimeValue>),
    QuasiQuotedList(Vec<EvauluatorRuntimeValue>),
    Symbol(String),
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Function {
        parameters: Vec<String>,
        body: Vec<EvauluatorRuntimeValue>,
    },
    Macro {
        parameters: Vec<String>,
        body: Box<EvauluatorRuntimeValue>,
    },
    BuiltIn(BuiltIn),
    CommaUnquote(Box<EvauluatorRuntimeValue>),
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
    // mirror of compiler::Literal
}

impl SrcSexpr {
    pub fn to_sexpr(&self) -> EvauluatorRuntimeValue {
        match self {
            SrcSexpr::List(sexprs) => EvauluatorRuntimeValue::List(sexprs.iter().map(|t| t.to_sexpr()).collect()),
            SrcSexpr::Symbol(s) => EvauluatorRuntimeValue::Symbol(s.clone()),
            SrcSexpr::String(s) => EvauluatorRuntimeValue::String(s.clone()),
            SrcSexpr::Bool(b) => EvauluatorRuntimeValue::Bool(*b),
            SrcSexpr::Int(i) => EvauluatorRuntimeValue::Int(*i),
            SrcSexpr::Float(f) => EvauluatorRuntimeValue::Float(*f),
            SrcSexpr::Quote(sexpr) => EvauluatorRuntimeValue::Quote(Box::new(sexpr.to_sexpr())),
        }
    }
}
