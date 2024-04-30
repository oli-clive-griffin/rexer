use crate::builtins::BuiltIn;

#[derive(Debug, PartialEq, Clone)]
pub enum Sexpr {
    List(Vec<Sexpr>),
    Quote(Box<Sexpr>),
    QuasiQuotedList(Vec<Sexpr>),
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

#[derive(Debug, PartialEq, Clone)]
pub enum SrcSexpr {
    List(Vec<SrcSexpr>),
    Quote(Box<SrcSexpr>),
    QuasiQuotedList(Vec<SrcSexpr>),
    Symbol(String),
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    CommaUnquote(Box<SrcSexpr>),
}

impl SrcSexpr {
    pub fn to_sexpr(&self) -> Sexpr {
        match self {
            SrcSexpr::List(sexprs) => {
                Sexpr::List(sexprs.iter().map(|t| t.to_sexpr()).collect())
            }
            SrcSexpr::Symbol(s) => Sexpr::Symbol(s.clone()),
            SrcSexpr::String(s) => Sexpr::String(s.clone()),
            SrcSexpr::Bool(b) => Sexpr::Bool(*b),
            SrcSexpr::Int(i) => Sexpr::Int(*i),
            SrcSexpr::Float(f) => Sexpr::Float(*f),
            SrcSexpr::CommaUnquote(t) => Sexpr::CommaUnquote(Box::new(t.to_sexpr())),
            SrcSexpr::Quote(sexpr) => Sexpr::Quote(Box::new(sexpr.to_sexpr())),
            SrcSexpr::QuasiQuotedList(sexprs) => {
                Sexpr::QuasiQuotedList(sexprs.iter().map(|t| t.to_sexpr()).collect())
            }
        }
    }
}
