use crate::vm::{CaptureType, Closure, ConstantObject};
use crate::{
    lexer, parser,
    sexpr::SrcSexpr,
    structural_parser::structure_ast,
    vm::{BytecodeChunk, ConstantValue, Function, Op},
};

// The goal is to get this to be `SrcSexpr`
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    // (if a b c)
    SrcSexpr(SrcSexpr),

    RegularForm(Vec<Expression>),

    /// (if condition then else)
    If {
        condition: Box<Expression>,
        then: Box<Expression>,
        else_: Box<Expression>,
    },

    /// (define name value)
    LocalDefine {
        name: String,
        value: Box<Expression>,
    },

    /// (set name value)
    LocalSet {
        name: String,
        value: Box<Expression>,
    },

    /// (define name value)
    DeclareGlobal {
        name: String,
        value: Box<Expression>,
    },

    /// An anonymous function
    /// (fn (args) ..body)
    FunctionLiteral(FunctionExpression),
    // Don't need to support for now
    // /// the value of `nil`
    // NilLit,
    Discard(Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionExpression {
    pub parameters: Vec<String>,
    pub body: Vec<Expression>,
    pub name: Option<String>,
}

impl FunctionExpression {
    pub fn new(parameters: Vec<String>, body: Vec<Expression>, name: Option<String>) -> Self {
        Self {
            parameters,
            body,
            name,
        }
    }
}

impl Into<ConstantValue> for SrcSexpr {
    fn into(self) -> ConstantValue {
        match self {
            SrcSexpr::Bool(x) => ConstantValue::Boolean(x),
            SrcSexpr::Int(x) => ConstantValue::Integer(x),
            SrcSexpr::Float(x) => ConstantValue::Float(x),
            SrcSexpr::String(x) => ConstantValue::Object(ConstantObject::String(x)),
            SrcSexpr::Symbol(x) => ConstantValue::Object(ConstantObject::Symbol(x)),
            SrcSexpr::List(l) => ConstantValue::List(l.into_iter().map(Into::into).collect()),
            SrcSexpr::Quote(x) => ConstantValue::Quote(Box::new((*x).into())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UpvalueCapture {
    Upvalue { i: usize },
    Local { i: usize },
    // capture_type: CaptureType,
    // /// local index in the enclosing scope
    // index: usize,
}

pub struct Compiler {
    chunks: Vec<ChunkCompiler>,
}
impl Compiler {
    fn new() -> Self {
        Compiler {
            chunks: vec![ChunkCompiler::new()],
        }
    }
}

#[derive(Debug)]
struct Local {
    name: String,
    // captured: bool,
}

impl Local {
    fn new(name: String) -> Self {
        Local {
            name,
            // captured: false,
        }
    }
}

pub struct ChunkCompiler {
    code: Vec<u8>,
    constants: Vec<ConstantValue>,
    args: Vec<Local>,
    locals: Vec<Local>,
    captured_upvalues: Vec<UpvalueCapture>,
}

impl ChunkCompiler {
    pub fn new() -> Self {
        ChunkCompiler {
            constants: vec![],
            args: vec![],
            locals: vec![],
            code: vec![],
            captured_upvalues: vec![],
        }
    }
}

impl Compiler {
    fn current_mut(&mut self) -> &mut ChunkCompiler {
        self.chunks.last_mut().unwrap()
    }

    fn current(&self) -> &ChunkCompiler {
        self.chunks.last().unwrap()
    }

    fn code_push(&mut self, op: u8) {
        self.current_mut().code.push(op);
    }

    fn compile_expression(&mut self, expression: Expression) {
        match expression {
            Expression::SrcSexpr(sexpr) => match sexpr {
                SrcSexpr::Symbol(sym) => {
                    self.compile_symbol_as_reference(sym);
                }
                SrcSexpr::Bool(_) | SrcSexpr::Int(_) | SrcSexpr::Float(_) | SrcSexpr::String(_) => {
                    self.compile_self_evaluation(sexpr, 0)
                }
                SrcSexpr::Quote(_) => self.compile_self_evaluation(sexpr, 1),
                SrcSexpr::List(_) => {
                    unreachable!("this should have been handled by the structural parser")
                }
            },
            Expression::If {
                condition,
                then,
                else_,
            } => self.compile_if_statement(condition, else_, then),
            Expression::RegularForm(exprs) => self.compile_regular_form(exprs),
            Expression::FunctionLiteral(function_expr) => {
                self.compile_function(function_expr);
            }
            Expression::DeclareGlobal { name, value } => {
                self.compile_global_declaration(name, value)
            }
            Expression::LocalDefine { name, value } => self.compile_local_definition(name, value),
            Expression::Discard(expr) => {
                match *expr {
                    Expression::LocalDefine { name: _, value: _ }
                    | Expression::DeclareGlobal { name: _, value: _ }
                    | Expression::Discard(_) => {
                        panic!("should not be discarding this expr: {:?}", expr)
                    }
                    _ => {}
                };

                self.compile_expression(*expr);
                self.code_push(Op::Pop.into());
            }
            Expression::LocalSet { name, value } => {
                self.compile_local_set(name, value);
            }
        }
    }

    fn compile_local_set(&mut self, sym: String, value: Box<Expression>) {
        self.compile_expression(*value);

        if let Some(idx) = self.resolve_local_pos(&sym, self.chunks.len() - 1) {
            self.code_push(Op::SetLocal.into());
            self.code_push((idx + 1) as u8);
        } else if let Some(upvalue_idx) = self.resolve_upvalue(&sym) {
            // relies on the fact that `self.resolve_upvalue` will populate the upvalue vec
            self.code_push(Op::SetUpvalue.into());
            self.code_push(upvalue_idx as u8);
        } else {
            panic!("can't set global yet")
        }
    }

    fn compile_function(&mut self, function_expr: FunctionExpression) {
        let ChunkCompiler {
            code,
            constants,
            captured_upvalues,
            args,
            locals,
        } = {
            self.chunks.push(ChunkCompiler {
                code: vec![],
                constants: vec![],
                args: function_expr
                    .parameters
                    .iter()
                    .map(|name| Local::new(name.clone()))
                    .collect(),
                locals: vec![],
                captured_upvalues: vec![],
            });
            for expr in function_expr.body {
                self.compile_expression(expr);
            }
            self.code_push(Op::Return.into());
            self.chunks.pop().unwrap()
        };

        let closure = Closure::new(
            Function::new(
                function_expr.name.unwrap_or("anonymous".to_string()),
                args.len(),
                locals.len(),
                BytecodeChunk::new(code, constants),
            ),
            captured_upvalues.len(),
        );
        self.code_push(Op::Closure.into());
        self.add_constant_and_push_idx(ConstantValue::Object(ConstantObject::Closure(closure)));
        for upvalue in captured_upvalues {
            match upvalue {
                UpvalueCapture::Local { i } => {
                    self.code_push(CaptureType::SurroundingLocal.into());
                    self.code_push(i as u8);
                }
                UpvalueCapture::Upvalue { i } => {
                    self.code_push(CaptureType::SurroundingUpvalue.into());
                    self.code_push(i as u8);
                }
            }
        }
    }

    fn compile_local_definition(&mut self, name: String, value: Box<Expression>) {
        let redefining_local = self
            .current()
            .args
            .iter()
            .chain(self.current().locals.iter())
            .all(|x| x.name != name);
        if !redefining_local {
            panic!("redefining local variable")
        };
        self.current_mut().locals.push(Local::new(name.clone()));
        self.compile_expression(*value);
        self.code_push(Op::Define.into());
        let idx = self.current().args.len() + self.current().locals.len();
        self.code_push(idx as u8);
    }

    fn compile_symbol_as_reference(&mut self, sym: String) {
        // evaulate as reference as opposed to value
        // local / function argument
        let chunk_idx = self.chunks.len() - 1;
        let local_idx = self.resolve_local_pos(&sym, chunk_idx);

        if let Some(idx) = local_idx {
            self.code_push(Op::ReferenceLocal.into());
            self.code_push((idx + 1) as u8);
        } else if let Some(upvalue_idx) = self.resolve_upvalue(&sym) {
            // relies on the fact that `self.resolve_upvalue` will populate the upvalue vec
            self.code_push(Op::ReferenceUpvalue.into());
            self.code_push(upvalue_idx as u8);
        } else {
            // fall back to global
            self.code_push(Op::ReferenceGlobal.into());
            // this can be optimized by reusing the same constant for the same symbol
            // also - this is one of those wierd/cool cases where a language concept becomes a runtime concept: the symbol in the code is a runtime value
            // can abstract this?
            self.add_constant_and_push_idx(ConstantValue::Object(ConstantObject::String(sym)));
        }
    }

    fn compile_regular_form(&mut self, exprs: Vec<Expression>) {
        // We don't know the arity of the function at compile-time so we
        // defensively put the number of arguments to check at runtime
        let arity = {
            let arity = exprs.len() - 1;
            if arity > 255 {
                panic!()
            }
            arity as u8
        };
        for expr in exprs {
            self.compile_expression(expr);
        }
        self.code_push(Op::FuncCall.into());
        self.code_push(arity);
    }

    fn compile_global_declaration(&mut self, name: String, value: Box<Expression>) {
        self.compile_expression(*value);
        self.code_push(Op::DeclareGlobal.into());
        self.current_mut()
            .constants
            .push(ConstantValue::Object(ConstantObject::String(name)));
        let idx = self.current().constants.len() as u8 - 1;
        self.code_push(idx);
    }

    fn compile_if_statement(
        &mut self,
        condition: Box<Expression>,
        else_: Box<Expression>,
        then: Box<Expression>,
    ) {
        // IF
        self.compile_expression(*condition);
        // skip to "then"
        self.code_push(Op::CondJump.into());
        self.code_push(0x00);
        // will mutate this later
        let then_jump_idx = self.current().code.len() - 1;
        // ELSE
        self.compile_expression(*else_);
        // skip to end
        self.code_push(Op::Jump.into());
        // self.current().code[to_then_jump_address as usize] = self.current().code.len() as u8;
        self.code_push(0x00);
        // will mutate this later
        let finish_jump_idx = self.current().code.len() - 1;
        // THEN
        let then_jump = (self.current().code.len() - then_jump_idx) as u8;
        self.current_mut().code[then_jump_idx] = then_jump;
        self.compile_expression(*then);
        // FINISH
        let finish_jump = (self.current().code.len() - finish_jump_idx) as u8;
        self.current_mut().code[finish_jump_idx] = finish_jump
    }

    fn compile_constant(&mut self, c: ConstantValue) {
        self.code_push(Op::Constant.into());
        self.add_constant_and_push_idx(c);
    }

    fn add_constant_and_push_idx(&mut self, c: ConstantValue) {
        self.current_mut().constants.push(c);
        let idx = self.current().constants.len() as u8 - 1;
        self.code_push(idx);
    }

    fn compile_self_evaluation(&mut self, sexpr: SrcSexpr, quote_level: usize) {
        match sexpr {
            SrcSexpr::Bool(x) => {
                self.compile_constant(ConstantValue::Boolean(x));
            }
            SrcSexpr::Int(x) => {
                self.compile_constant(ConstantValue::Integer(x));
            }
            SrcSexpr::Float(x) => {
                self.compile_constant(ConstantValue::Float(x));
            }
            SrcSexpr::String(x) => {
                self.compile_constant(ConstantValue::Object(ConstantObject::String(x)));
            }
            SrcSexpr::Symbol(x) => {
                self.compile_constant(ConstantValue::Object(ConstantObject::Symbol(x)));
            }
            // NOTE this is a literal sexpr list: `'()`, not a list constructor: `(list 1 2 3)`. The latter is a regular form
            SrcSexpr::List(sexprs) => {
                let const_sexprs = sexprs.into_iter().map(Into::into).collect();

                self.compile_constant(ConstantValue::List(const_sexprs))
            }
            SrcSexpr::Quote(quoted_sexpr) => {
                self.compile_self_evaluation(*quoted_sexpr, quote_level + 1);
                // this is kinda hacky:
                // The first level of quoting is handled by the compiler, by compiling, for example, `'x` to the literal symbol `x`
                // in the case of `''x`, the first quote is handled by the compiler, and the second quote is handled here:
                if quote_level >= 2 {
                    self.code_push(Op::Quote.into());
                }
            }
        }
    }

    fn resolve_upvalue(&mut self, sym: &str) -> Option<usize> {
        match self.chunks.len() {
            0 => panic!("no chunks"),
            1 => None,
            two_or_more => self.resolve_upvalue_rec(sym, two_or_more - 1), // start at the top
        }
    }

    fn resolve_upvalue_rec(&mut self, sym: &str, chunk_idx: usize) -> Option<usize> {
        if chunk_idx == 0 {
            return None;
        }

        if let Some(local_index) = self.resolve_local_pos(sym, chunk_idx - 1) {
            // mark the local as captured
            // self.chunks.get_mut(chunk_idx - 1).unwrap().locals[local_index].captured = true;
            let upvalue_idx = self.add_upvalue(UpvalueCapture::Local { i: local_index }, chunk_idx);
            return Some(upvalue_idx);
        };

        if let Some(upvalue_index) = self.resolve_upvalue_rec(sym, chunk_idx - 1) {
            let upvalue_idx =
                self.add_upvalue(UpvalueCapture::Upvalue { i: upvalue_index }, chunk_idx);
            return Some(upvalue_idx);
        };

        None
    }

    fn resolve_local_pos(&self, sym: &str, chunk_idx: usize) -> Option<usize> {
        let compiler = self.chunks.get(chunk_idx).unwrap();
        return compiler
            .args
            .iter()
            .chain(compiler.locals.iter())
            .position(|x| x.name == sym);
    }

    fn add_upvalue(&mut self, uv: UpvalueCapture, chunk_index: usize) -> usize {
        let compiler = self.chunks.get_mut(chunk_index).unwrap();
        if let Some(upvalue_idx) = compiler.captured_upvalues.iter().position(|x| x == &uv) {
            return upvalue_idx;
        }

        compiler.captured_upvalues.push(uv);
        compiler.captured_upvalues.len() - 1
    }
}

pub fn compile(src: &String) -> BytecodeChunk {
    let tokens = lexer::lex(src).unwrap_or_else(|e| {
        panic!("Lexing error: {}", e);
    });
    // println!("{:#?}", tokens);

    let ast = parser::parse(tokens).unwrap_or_else(|e| {
        panic!("Parsing error: {}", e);
    });
    // println!("{:#?}", ast);

    let expressions = structure_ast(ast);
    // println!("{:#?}", expressions);

    compile_expressions(expressions)
}

fn compile_expressions(expressions: Vec<Expression>) -> BytecodeChunk {
    let mut compiler = Compiler::new();

    for expression in expressions {
        compiler.compile_expression(expression);
    }
    compiler.current_mut().code.push(Op::DebugEnd.into());

    if compiler.chunks.len() != 1 {
        panic!("There should only be one chunk at the end of compilation")
    }

    let comp = compiler.chunks.pop().unwrap();

    BytecodeChunk::new(comp.code, comp.constants)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::vm::{SmallVal, VM};

    use super::*;

    #[test]
    fn test_if() {
        let expression = Expression::If {
            condition: Box::new(Expression::SrcSexpr(SrcSexpr::Int(11))),
            then: Box::new(Expression::SrcSexpr(SrcSexpr::Int(12))),
            else_: Box::new(Expression::SrcSexpr(SrcSexpr::Int(13))),
        };
        let bc = compile_expressions(vec![expression]);
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::CondJump.into(),
                5,
                Op::Constant.into(),
                1,
                Op::Jump.into(),
                3,
                Op::Constant.into(),
                2,
                Op::DebugEnd.into(),
            ]
        );
        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Integer(11),
                ConstantValue::Integer(13),
                ConstantValue::Integer(12)
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.at(0).unwrap(), &SmallVal::Integer(12));
    }

    #[test]
    fn test_declare_global() {
        let expression = Expression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(Expression::SrcSexpr(SrcSexpr::Int(11))),
        };
        let bc = compile_expressions(vec![expression]);
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::DeclareGlobal.into(),
                1,
                Op::DebugEnd.into(),
            ]
        );
        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Integer(11),
                ConstantValue::Object(ConstantObject::String("foo".to_string())),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.globals.get("foo"), Some(&SmallVal::Integer(11)))
    }

    #[test]
    fn test_assign_global_as_expr() {
        let program = vec![
            Expression::DeclareGlobal {
                name: "foo".to_string(),
                value: Box::new(Expression::RegularForm(vec![
                    Expression::SrcSexpr(SrcSexpr::Symbol("+".to_string())),
                    Expression::SrcSexpr(SrcSexpr::Int(11)),
                    Expression::SrcSexpr(SrcSexpr::Int(12)),
                ])),
            },
            Expression::SrcSexpr(SrcSexpr::Symbol("foo".to_string())),
        ];

        let bc = compile_expressions(program);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ConstantObject::String("+".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
                ConstantValue::Object(ConstantObject::String("foo".to_string())),
                ConstantValue::Object(ConstantObject::String("foo".to_string())),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::ReferenceGlobal.into(),
                0, // "+"
                Op::Constant.into(),
                1, // "11"
                Op::Constant.into(),
                2, // "12"
                Op::FuncCall.into(),
                2, // arity
                Op::DeclareGlobal.into(),
                3, // "foo" constant index
                Op::ReferenceGlobal.into(),
                4, // "foo" constant index
                Op::DebugEnd.into(),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.globals.get("foo"), Some(&SmallVal::Integer(23)));
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.at(0).unwrap(), &SmallVal::Integer(23));
    }

    #[test]
    fn test_call_function() {
        let bc = compile_expressions(vec![Expression::RegularForm(vec![
            Expression::SrcSexpr(SrcSexpr::Symbol("*".to_string())),
            Expression::SrcSexpr(SrcSexpr::Int(11)),
            Expression::SrcSexpr(SrcSexpr::Int(12)),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ConstantObject::String("*".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
            ],
        );

        assert_eq!(
            bc.code,
            vec![
                Op::ReferenceGlobal.into(),
                0, // load function symbol
                Op::Constant.into(),
                1, // load arg 1
                Op::Constant.into(),
                2, // load arg 2
                Op::FuncCall.into(),
                2, // call function with arity 2
                Op::DebugEnd.into(),
            ]
        );
    }

    #[test]
    fn test_function_with_computed_arguments() {
        let bc = compile_expressions(vec![Expression::RegularForm(vec![
            Expression::SrcSexpr(SrcSexpr::Symbol("+".to_string())),
            Expression::RegularForm(vec![
                Expression::SrcSexpr(SrcSexpr::Symbol("+".to_string())),
                Expression::SrcSexpr(SrcSexpr::Int(11)),
                Expression::SrcSexpr(SrcSexpr::Int(12)),
            ]),
            Expression::RegularForm(vec![
                Expression::SrcSexpr(SrcSexpr::Symbol("*".to_string())),
                Expression::SrcSexpr(SrcSexpr::Int(13)),
                Expression::SrcSexpr(SrcSexpr::Int(14)),
            ]),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ConstantObject::String("+".to_string())),
                ConstantValue::Object(ConstantObject::String("+".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
                ConstantValue::Object(ConstantObject::String("*".to_string())),
                ConstantValue::Integer(13),
                ConstantValue::Integer(14),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::ReferenceGlobal.into(),
                0, // reference function symbol (outer)
                Op::ReferenceGlobal.into(),
                1, // reference function symbol (inner)
                Op::Constant.into(),
                2, // load arg 1: "11"
                Op::Constant.into(),
                3, // load arg 2: "12"
                Op::FuncCall.into(),
                2, // call inner "+" with arity 2
                Op::ReferenceGlobal.into(),
                4, // load "*"
                Op::Constant.into(),
                5, // load arg 1: "13"
                Op::Constant.into(),
                6, // load arg 2: "14"
                Op::FuncCall.into(),
                2, // call function "*" (outer) with arity 2
                Op::FuncCall.into(),
                2, // call function "+" (outer) with arity 2
                Op::DebugEnd.into(),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.at(0).unwrap(), &SmallVal::Integer(205));
    }
}
