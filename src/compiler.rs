use crate::{
    lexer,
    parser::{self, Ast},
    sexpr::SrcSexpr,
    structural_parser::structure_sexpr,
    vm::{BytecodeChunk, ConstantValue, Function, ObjectValue, Op},
};

// The goal is to get this to be `SrcSexpr`
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    SrcSexpr(SrcSexpr),
    RegularForm(Vec<Expression>),

    /// (if condition then else)
    If {
        condition: Box<Expression>,
        then: Box<Expression>,
        else_: Box<Expression>,
    },

    /// (define name value)
    Define {
        name: String,
        value: Box<Expression>,
    },

    /// (define name value)
    DeclareGlobal {
        name: String,
        value: Box<Expression>,
    },

    /// (defun (name args) ..body)
    GlobalFunctionDeclaration {
        name: String,
        function_expr: FunctionExpression,
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
}

impl FunctionExpression {
    pub fn new(parameters: Vec<String>, body: Vec<Expression>) -> Self {
        Self { parameters, body }
    }
}

fn compile_expression(
    expression: Expression,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
    locals: &mut Vec<String>,
) {
    match expression {
        Expression::SrcSexpr(sexpr) => {
            match sexpr {
                SrcSexpr::Bool(b) => {
                    constants.push(ConstantValue::Boolean(b));
                    code.push(Op::Constant.into());
                    code.push(constants.len() as u8 - 1);
                }
                SrcSexpr::Integer(i) => {
                    constants.push(ConstantValue::Integer(i));
                    code.push(Op::Constant.into());
                    code.push(constants.len() as u8 - 1);
                }
                SrcSexpr::Float(f) => {
                    constants.push(ConstantValue::Float(f));
                    code.push(Op::Constant.into());
                    code.push(constants.len() as u8 - 1);
                }
                SrcSexpr::String(s) => {
                    constants.push(ConstantValue::Object(ObjectValue::String(s)));
                    code.push(Op::Constant.into());
                    code.push(constants.len() as u8 - 1);
                }
                SrcSexpr::Symbol(s) => {
                    // local / function argument
                    let local_idx = locals.iter().position(|x| x == &s);
                    if let Some(idx) = local_idx {
                        code.push(Op::ReferenceLocal.into());
                        code.push((idx + 1) as u8); // plus 1 because locals are 1-indexed
                        return;
                    }

                    // fall back to global
                    code.push(Op::ReferenceGlobal.into());
                    // this can be optimized by reusing the same constant for the same symbol
                    // also - this is one of those wierd/cool cases where a language concept becomes a runtime concept: the symbol in the code is a runtime value
                    constants.push(ConstantValue::Object(ObjectValue::String(s)));
                    code.push(constants.len() as u8 - 1);
                }
                SrcSexpr::Quote(expr) => compile_quoted_sexpr(*expr, code, constants, locals),
                SrcSexpr::List(_list) => {
                    unreachable!("List should be handled by RegularForm")
                }
            }
        }
        Expression::If {
            condition,
            then,
            else_,
        } => {
            // IF
            compile_expression(*condition, code, constants, locals);

            // skip to "then"
            code.push(Op::CondJump.into());
            code.push(0x00); // will mutate this later
            let then_jump_idx = code.len() - 1;

            // ELSE
            compile_expression(*else_, code, constants, locals);

            // skip to end
            code.push(Op::Jump.into());
            // code[to_then_jump_address as usize] = code.len() as u8;
            code.push(0x00); // will mutate this later
            let finish_jump_idx = code.len() - 1;

            // THEN
            let then_jump = (code.len() - then_jump_idx) as u8;
            code[then_jump_idx] = then_jump;
            compile_expression(*then, code, constants, locals);

            // FINISH
            let finish_jump = (code.len() - finish_jump_idx) as u8;
            code[finish_jump_idx] = finish_jump
        }
        Expression::DeclareGlobal { name, value } => {
            compile_expression(*value, code, constants, locals);
            code.push(Op::DeclareGlobal.into());
            constants.push(ConstantValue::Object(ObjectValue::String(name)));
            code.push(constants.len() as u8 - 1);
        }
        Expression::RegularForm(exprs) => {
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
                compile_expression(expr, code, constants, locals);
            }

            code.push(Op::FuncCall.into());
            code.push(arity);
        }
        // Expression::Quote(sexpr) => compile_quoted_sexpr(sexpr, code, constants, locals),
        // Expression::QuasiQuotedList(_sexprs) => {
        //     todo!()
        // }
        Expression::GlobalFunctionDeclaration {
            name,
            function_expr,
        } => {
            // potenially jank way of doing it for now:

            // make function object
            let function_obj = ConstantValue::Object(ObjectValue::Function(compile_function(
                Some(name.clone()),
                function_expr,
            )));

            // reference constant function object on stack
            code.push(Op::Constant.into());
            constants.push(function_obj);
            code.push(constants.len() as u8 - 1);

            // declare top of stack as global
            code.push(Op::DeclareGlobal.into());
            constants.push(ConstantValue::Object(ObjectValue::String(name)));
            code.push(constants.len() as u8 - 1);
        }
        Expression::Define { name: _, value: _ } => {
            todo!()
        }
        Expression::FunctionLiteral(function_expr) => {
            compile_function(None, function_expr);
        }
    }
}

fn compile_quoted_sexpr(
    sexpr: SrcSexpr,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
    locals: &mut Vec<String>,
) {
    match sexpr {
        SrcSexpr::Quote(_expr) => todo!(), // nested quotes might get weird?
        SrcSexpr::List(sexprs) => {
            // nil for end of list
            code.push(Op::Constant.into());
            constants.push(ConstantValue::Nil);
            code.push(constants.len() as u8 - 1);

            // cons each element in reverse order
            for expr in sexprs.iter().rev() {
                compile_expression(
                    Expression::SrcSexpr(SrcSexpr::Quote(Box::new(expr.clone()))),
                    code,
                    constants,
                    locals,
                );
                code.push(Op::Cons.into())
            }
        }
        //
        // TODO these should be doable in a better way
        SrcSexpr::Symbol(s) => compile_expression(
            Expression::SrcSexpr(SrcSexpr::Symbol(s)),
            code,
            constants,
            locals,
        ),
        SrcSexpr::Integer(i) => compile_expression(
            Expression::SrcSexpr(SrcSexpr::Integer(i)),
            code,
            constants,
            locals,
        ),
        SrcSexpr::Float(f) => compile_expression(
            Expression::SrcSexpr(SrcSexpr::Float(f)),
            code,
            constants,
            locals,
        ),
        SrcSexpr::String(s) => compile_expression(
            Expression::SrcSexpr(SrcSexpr::String(s)),
            code,
            constants,
            locals,
        ),
        SrcSexpr::Bool(b) => compile_expression(
            Expression::SrcSexpr(SrcSexpr::Bool(b)),
            code,
            constants,
            locals,
        ),
    }
}

fn compile_function(name: Option<String>, function_expr: FunctionExpression) -> Function {
    let mut code = vec![];
    let mut constants = vec![];
    let mut locals = function_expr.parameters.clone();

    for expr in function_expr.body {
        compile_expression(expr, &mut code, &mut constants, &mut locals);
    }

    code.push(Op::Return.into());

    Function::new(
        name.unwrap_or("anonymous".to_string()),
        function_expr.parameters.len(),
        BytecodeChunk::new(code, constants),
    )
}

fn compile_ast(ast: Ast) -> BytecodeChunk {
    // let sexprs = macro_expand(sexprs);
    let expressions = ast
        .expressions
        .iter()
        .map(structure_sexpr)
        .collect::<Vec<Expression>>();

    compile_expressions(expressions)
}

fn compile_expressions(expressions: Vec<Expression>) -> BytecodeChunk {
    let mut code: Vec<u8> = vec![];
    let mut constants: Vec<ConstantValue> = vec![];
    let mut locals: Vec<String> = vec![];
    for expression in expressions {
        compile_expression(expression, &mut code, &mut constants, &mut locals);
    }
    code.push(Op::DebugEnd.into());
    BytecodeChunk::new(code, constants)
}

pub fn compile(src: &String) -> BytecodeChunk {
    let tokens = lexer::lex(src).unwrap_or_else(|e| {
        eprintln!("Lexing error: {}", e);
        std::process::exit(1);
    });

    let ast = parser::parse(tokens).unwrap_or_else(|e| {
        eprintln!("Parsing error: {}", e);
        std::process::exit(1);
    });

    compile_ast(ast)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        static_stack::StaticStack,
        vm::{SmallValue, VM},
    };

    use super::*;

    #[test]
    fn test_if() {
        let expression = Expression::If {
            condition: Box::new(Expression::SrcSexpr(SrcSexpr::Integer(11))),
            then: Box::new(Expression::SrcSexpr(SrcSexpr::Integer(12))),
            else_: Box::new(Expression::SrcSexpr(SrcSexpr::Integer(13))),
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
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(12)]))
    }

    #[test]
    fn test_declare_global() {
        let expression = Expression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(Expression::SrcSexpr(SrcSexpr::Integer(11))),
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
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.globals.get("foo"), Some(&SmallValue::Integer(11)))
    }

    #[test]
    fn test_assign_global_as_expr() {
        let program = vec![
            Expression::DeclareGlobal {
                name: "foo".to_string(),
                value: Box::new(Expression::RegularForm(vec![
                    Expression::SrcSexpr(SrcSexpr::Symbol("+".to_string())),
                    Expression::SrcSexpr(SrcSexpr::Integer(11)),
                    Expression::SrcSexpr(SrcSexpr::Integer(12)),
                ])),
            },
            Expression::SrcSexpr(SrcSexpr::Symbol("foo".to_string())),
        ];

        let bc = compile_expressions(program);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ObjectValue::String("+".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
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
        assert_eq!(vm.globals.get("foo"), Some(&SmallValue::Integer(23)));
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(23)]))
    }

    #[test]
    fn test_call_function() {
        let bc = compile_expressions(vec![Expression::RegularForm(vec![
            Expression::SrcSexpr(SrcSexpr::Symbol("*".to_string())),
            Expression::SrcSexpr(SrcSexpr::Integer(11)),
            Expression::SrcSexpr(SrcSexpr::Integer(12)),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ObjectValue::String("*".to_string())),
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
                Expression::SrcSexpr(SrcSexpr::Integer(11)),
                Expression::SrcSexpr(SrcSexpr::Integer(12)),
            ]),
            Expression::RegularForm(vec![
                Expression::SrcSexpr(SrcSexpr::Symbol("*".to_string())),
                Expression::SrcSexpr(SrcSexpr::Integer(13)),
                Expression::SrcSexpr(SrcSexpr::Integer(14)),
            ]),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ObjectValue::String("+".to_string())),
                ConstantValue::Object(ObjectValue::String("+".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
                ConstantValue::Object(ObjectValue::String("*".to_string())),
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
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(205)]));
    }
    #[test]
    fn test_cons() {
        let bc = compile_expressions(vec![Expression::SrcSexpr(SrcSexpr::Quote(Box::new(
            SrcSexpr::List(vec![
                SrcSexpr::Integer(1),
                SrcSexpr::Integer(2),
                SrcSexpr::Integer(3),
            ]),
        )))]);

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
        let bc = compile_expressions(vec![Expression::SrcSexpr(SrcSexpr::Quote(Box::new(
            SrcSexpr::List(vec![
                SrcSexpr::Integer(10),
                SrcSexpr::List(vec![SrcSexpr::Integer(20), SrcSexpr::Integer(30)]),
                SrcSexpr::Integer(40),
            ]),
        )))]);

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
