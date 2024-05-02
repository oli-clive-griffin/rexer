use crate::vm::ConstantObject;
use crate::{
    lexer,
    parser::{self, Ast},
    sexpr::SrcSexpr,
    structural_parser::structure_sexpr,
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionExpression {
    parameters: Vec<String>,
    body: Vec<Expression>,
    name: Option<String>,
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

pub struct ChunkCompiler<'a> {
    code: Vec<u8>,
    constants: Vec<ConstantValue>,
    local_and_arg_tracker: Vec<String>,
    parent: Option<Box<&'a ChunkCompiler<'a>>>,
}

impl<'a> ChunkCompiler<'a> {
    fn compile_expression(self: &mut Self, expression: Expression) {
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
                let f = ChunkCompiler::with_parent(self).compile_function(function_expr);

                self.code.push(Op::Constant.into());
                self.constants
                    .push(ConstantValue::Object(ConstantObject::Function(f)));
                self.code.push(self.constants.len() as u8 - 1);
            }
            Expression::DeclareGlobal { name, value } => {
                self.compile_global_declaration(name, value)
            }
            Expression::LocalDefine { name, value } => self.compile_local_definition(name, value),
        }
    }

    fn compile_local_definition(self: &mut Self, name: String, value: Box<Expression>) {
        let legal = self.local_and_arg_tracker.iter().all(|x| x != &name);
        if !legal {
            panic!("redefining local variable")
        };
        self.local_and_arg_tracker.push(name.clone());
        self.compile_expression(*value);
        self.code.push(Op::Define.into());
        self.code.push(self.local_and_arg_tracker.len() as u8);
    }

    fn compile_symbol_as_reference(self: &mut Self, sym: String) {
        // evaulate as reference as opposed to value
        // local / function argument
        let local_idx = self.local_and_arg_tracker.iter().position(|x| x == &sym);
        if let Some(idx) = local_idx {
            self.code.push(Op::ReferenceLocal.into());
            self.code.push((idx + 1) as u8); // plus 1 because local_and_arg_tracker are 1-indexed
        } else
        // if let Some(parent) = &self.parent {

        // }
        {
            // fall back to global
            self.code.push(Op::ReferenceGlobal.into());
            // this can be optimized by reusing the same constant for the same symbol
            // also - this is one of those wierd/cool cases where a language concept becomes a runtime concept: the symbol in the code is a runtime value
            self.constants
                .push(ConstantValue::Object(ConstantObject::String(sym)));
            self.code.push(self.constants.len() as u8 - 1);
        }
    }

    fn compile_regular_form(self: &mut Self, exprs: Vec<Expression>) {
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
        self.code.push(Op::FuncCall.into());
        self.code.push(arity);
    }

    fn compile_global_declaration(self: &mut Self, name: String, value: Box<Expression>) {
        self.compile_expression(*value);
        self.code.push(Op::DeclareGlobal.into());
        self.constants
            .push(ConstantValue::Object(ConstantObject::String(name)));
        self.code.push(self.constants.len() as u8 - 1);
    }

    fn compile_if_statement(
        self: &mut Self,
        condition: Box<Expression>,
        else_: Box<Expression>,
        then: Box<Expression>,
    ) {
        // IF
        self.compile_expression(*condition);
        // skip to "then"
        self.code.push(Op::CondJump.into());
        self.code.push(0x00);
        // will mutate this later
        let then_jump_idx = self.code.len() - 1;
        // ELSE
        self.compile_expression(*else_);
        // skip to end
        self.code.push(Op::Jump.into());
        // self.code[to_then_jump_address as usize] = self.code.len() as u8;
        self.code.push(0x00);
        // will mutate this later
        let finish_jump_idx = self.code.len() - 1;
        // THEN
        let then_jump = (self.code.len() - then_jump_idx) as u8;
        self.code[then_jump_idx] = then_jump;
        self.compile_expression(*then);
        // FINISH
        let finish_jump = (self.code.len() - finish_jump_idx) as u8;
        self.code[finish_jump_idx] = finish_jump
    }

    fn compile_self_evaluation(self: &mut Self, sexpr: SrcSexpr, quote_level: usize) {
        match sexpr {
            SrcSexpr::Bool(x) => {
                self.code.push(Op::Constant.into());
                self.constants.push(ConstantValue::Boolean(x));
                self.code.push(self.constants.len() as u8 - 1);
            }
            SrcSexpr::Int(x) => {
                self.code.push(Op::Constant.into());
                self.constants.push(ConstantValue::Integer(x));
                self.code.push(self.constants.len() as u8 - 1);
            }
            SrcSexpr::Float(x) => {
                self.code.push(Op::Constant.into());
                self.constants.push(ConstantValue::Float(x));
                self.code.push(self.constants.len() as u8 - 1);
            }
            SrcSexpr::String(x) => {
                self.code.push(Op::Constant.into());
                self.constants
                    .push(ConstantValue::Object(ConstantObject::String(x)));
                self.code.push(self.constants.len() as u8 - 1);
            }
            SrcSexpr::Symbol(x) => {
                self.code.push(Op::Constant.into());
                self.constants
                    .push(ConstantValue::Object(ConstantObject::Symbol(x)));
                self.code.push(self.constants.len() as u8 - 1);
            }
            // NOTE this is a literal sexpr list: `'()`, not a list constructor: `(list 1 2 3)`. The latter is a regular form
            SrcSexpr::List(sexprs) => {
                // nil for end of list
                self.code.push(Op::Constant.into());
                self.constants.push(ConstantValue::Nil);
                self.code.push(self.constants.len() as u8 - 1);

                // cons each element in reverse order
                for sexpr in sexprs.into_iter().rev() {
                    self.compile_self_evaluation(sexpr, quote_level);
                    self.code.push(Op::Cons.into());
                }
            }
            SrcSexpr::Quote(quoted_sexpr) => {
                self.compile_self_evaluation(*quoted_sexpr, quote_level + 1);
                // this is kinda hacky:
                // The first level of quoting is handled by the compiler, by compiling, for example, `'x` to the literal symbol `x`
                // in the case of `''x`, the first quote is handled by the compiler, and the second quote is handled here:
                if quote_level >= 2 {
                    self.code.push(Op::Quote.into());
                }
            }
        }
    }

    // sibling function to `compile_expressions`
    fn compile_function(mut self: Self, function_expr: FunctionExpression) -> Function {
        self.local_and_arg_tracker = function_expr.parameters.clone();

        let arity = function_expr.parameters.len();

        for expr in function_expr.body {
            self.compile_expression(expr);
        }

        self.code.push(Op::Return.into());
        let num_locals = self.local_and_arg_tracker.len() - arity;

        Function::new(
            function_expr.name.unwrap_or("anonymous".to_string()),
            arity,
            num_locals as usize,
            BytecodeChunk::new(self.code, self.constants),
        )
    }

    fn compile_ast(mut self: Self, ast: Ast) -> BytecodeChunk {
        // let sexprs = macro_expand(sexprs);
        let expressions = ast
            .expressions
            .iter()
            .map(|s| structure_sexpr(s, false)) // top-level isn't in a function
            .collect::<Vec<Expression>>();

        // println!("{:#?}", expressions);

        self.compile_expressions(expressions)
    }

    fn compile_expressions(mut self: Self, expressions: Vec<Expression>) -> BytecodeChunk {
        for expression in expressions {
            self.compile_expression(expression);
        }
        self.code.push(Op::DebugEnd.into());

        BytecodeChunk::new(self.code, self.constants)
    }

    pub fn compile(mut self: Self, src: &String) -> BytecodeChunk {
        let tokens = lexer::lex(src).unwrap_or_else(|e| {
            panic!("Lexing error: {}", e);
        });
        // println!("{:#?}", tokens);

        let ast = parser::parse(tokens).unwrap_or_else(|e| {
            panic!("Parsing error: {}", e);
        });
        // println!("{:#?}", ast);

        self.compile_ast(ast)
    }

    pub fn new() -> Self {
        ChunkCompiler {
            constants: vec![],
            local_and_arg_tracker: vec![],
            parent: None,
            code: vec![],
        }
    }

    fn with_parent(parent: &'a Self) -> Self {
        ChunkCompiler {
            constants: vec![],
            local_and_arg_tracker: vec![],
            parent: Some(Box::new(parent)),
            code: vec![],
        }
    }

    // fn for_function(parent: &'a Self, function_expr: FunctionExpression) -> Self {
    //     ChunkCompiler {
    //         constants: vec![],
    //         local_and_arg_tracker: function_expr.parameters.clone(),
    //         parent: Some(Box::new(parent)),
    //         code: vec![],
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        static_stack::StaticStack,
        vm::{SmallVal, VM},
    };

    use super::*;

    #[test]
    fn test_if() {
        let expression = Expression::If {
            condition: Box::new(Expression::SrcSexpr(SrcSexpr::Int(11))),
            then: Box::new(Expression::SrcSexpr(SrcSexpr::Int(12))),
            else_: Box::new(Expression::SrcSexpr(SrcSexpr::Int(13))),
        };
        let bc = ChunkCompiler::new().compile_expressions(vec![expression]);
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
        assert_eq!(vm.stack, StaticStack::from([SmallVal::Integer(12)]))
    }

    #[test]
    fn test_declare_global() {
        let expression = Expression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(Expression::SrcSexpr(SrcSexpr::Int(11))),
        };
        let bc = ChunkCompiler::new().compile_expressions(vec![expression]);
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

        let bc = ChunkCompiler::new().compile_expressions(program);

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
        assert_eq!(vm.stack, StaticStack::from([SmallVal::Integer(23)]))
    }

    #[test]
    fn test_call_function() {
        let bc = ChunkCompiler::new().compile_expressions(vec![Expression::RegularForm(vec![
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
        let bc = ChunkCompiler::new().compile_expressions(vec![Expression::RegularForm(vec![
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
        assert_eq!(vm.stack, StaticStack::from([SmallVal::Integer(205)]));
    }
    #[test]
    fn test_cons() {
        let bc = ChunkCompiler::new().compile_expressions(vec![Expression::SrcSexpr(
            SrcSexpr::Quote(Box::new(SrcSexpr::List(vec![
                SrcSexpr::Int(1),
                SrcSexpr::Int(2),
                SrcSexpr::Int(3),
            ]))),
        )]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Nil,
                ConstantValue::Integer(3),
                ConstantValue::Integer(2),
                ConstantValue::Integer(1),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0, // load nil
                Op::Constant.into(),
                1, // load 3
                Op::Cons.into(),
                Op::Constant.into(),
                2, // load 2
                Op::Cons.into(),
                Op::Constant.into(),
                3, // load 1
                Op::Cons.into(),
                Op::DebugEnd.into(),
            ]
        );
    }

    #[test]
    fn test_cons_nested() {
        let bc = ChunkCompiler::new().compile_expressions(vec![Expression::SrcSexpr(
            SrcSexpr::Quote(Box::new(SrcSexpr::List(vec![
                SrcSexpr::Int(10),
                SrcSexpr::List(vec![SrcSexpr::Int(20), SrcSexpr::Int(30)]),
                SrcSexpr::Int(40),
            ]))),
        )]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Nil,
                ConstantValue::Integer(40),
                ConstantValue::Nil,
                ConstantValue::Integer(30),
                ConstantValue::Integer(20),
                ConstantValue::Integer(10),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0, // load nil
                Op::Constant.into(),
                1, // load 4
                Op::Cons.into(),
                Op::Constant.into(),
                2, // load nil
                Op::Constant.into(),
                3, // load 3
                Op::Cons.into(),
                Op::Constant.into(),
                4, // load 2
                Op::Cons.into(),
                Op::Cons.into(),
                Op::Constant.into(),
                5, // load 1
                Op::Cons.into(),
                Op::DebugEnd.into(),
            ]
        );
    }
}
