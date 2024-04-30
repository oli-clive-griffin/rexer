use crate::builtins::BuiltIn;

#[derive(Debug, PartialEq, Clone)]
/// Previously known as `Sexpr`
pub enum RuntimeExpr {
    List(Vec<RuntimeExpr>),
    Quote(Box<RuntimeExpr>),
    QuasiQuotedList(Vec<RuntimeExpr>),
    Symbol(String),
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Function {
        parameters: Vec<String>,
        body: Vec<RuntimeExpr>,
    },
    Macro {
        parameters: Vec<String>,
        body: Box<RuntimeExpr>,
    },
    BuiltIn(BuiltIn),
    CommaUnquote(Box<RuntimeExpr>),
    Nil,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SrcSexpr {
    Bool(bool), // true, false
    Integer(i64), // 1
    Float(f64), // 1.0
    String(String), // "foo"
    Symbol(String), // +, -, *, /, foo
    List(Vec<SrcSexpr>), // (+ 2 3)
    Quote(Box<SrcSexpr>), // '(+ 2 3), 'foo
    // mirror of compiler::Literal
}

// #[derive(Debug, PartialEq, Clone)]
// pub enum Sexpr {
//     //                     // sexpr              evaluated
//     //                     // -------------------------
//     Bool(bool),     //     // true            -> true
//     Int(i64),       //     // 3               -> 3
//     Float(f64),     //     // 0.1             -> 0.1
//     String(String), //     // "foo"           -> "foo"
//     Symbol(String), //     // foo             -> <whatever foo is>
//     //                     // +               -> <builtin function +>
//     Quote(Box<Sexpr>), //  // 'foo            -> foo
//     //                     // '"asdf"         -> "asdf"
//     //                     // '3              -> 3
//     List(Vec<Expression>), // '(1 "foo" 'bar) -> (1 "foo" 'bar)
// }

impl SrcSexpr {
    pub fn to_sexpr(&self) -> RuntimeExpr {
        match self {
            SrcSexpr::List(sexprs) => RuntimeExpr::List(sexprs.iter().map(|t| t.to_sexpr()).collect()),
            SrcSexpr::Symbol(s) => RuntimeExpr::Symbol(s.clone()),
            SrcSexpr::String(s) => RuntimeExpr::String(s.clone()),
            SrcSexpr::Bool(b) => RuntimeExpr::Bool(*b),
            SrcSexpr::Integer(i) => RuntimeExpr::Int(*i),
            SrcSexpr::Float(f) => RuntimeExpr::Float(*f),
            SrcSexpr::Quote(sexpr) => RuntimeExpr::Quote(Box::new(sexpr.to_sexpr())),
        }
    }
}
