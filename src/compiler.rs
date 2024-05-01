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
        Expression::SrcSexpr(sexpr) => match sexpr {
            SrcSexpr::Symbol(sym) => {
                // evaulate as reference as opposed to value
                // local / function argument
                let local_idx = locals.iter().position(|x| x == &sym);
                if let Some(idx) = local_idx {
                    code.push(Op::ReferenceLocal.into());
                    code.push((idx + 1) as u8); // plus 1 because locals are 1-indexed
                    return;
                }
                // fall back to global
                code.push(Op::ReferenceGlobal.into());
                // this can be optimized by reusing the same constant for the same symbol
                // also - this is one of those wierd/cool cases where a language concept becomes a runtime concept: the symbol in the code is a runtime value
                constants.push(ConstantValue::Object(ConstantObject::String(sym)));
                code.push(constants.len() as u8 - 1);
            }
            SrcSexpr::Bool(_) | SrcSexpr::Int(_) | SrcSexpr::Float(_) | SrcSexpr::String(_) => {
                compile_self_evaluation(sexpr, code, constants, 0)
            }
            SrcSexpr::Quote(_) => compile_self_evaluation(sexpr, code, constants, 1),
            SrcSexpr::List(_) => {
                unreachable!("this should have been handled by the structural parser")
            }
        },
        Expression::If {
            condition,
            then,
            else_,
        } => compile_if_statement(condition, code, constants, locals, else_, then),
        Expression::DeclareGlobal { name, value } => {
            compile_global_declaration(value, code, constants, locals, name)
        }
        Expression::RegularForm(exprs) => compile_regular_form(exprs, code, constants, locals),
        Expression::GlobalFunctionDeclaration {
            name,
            function_expr,
        } => compile_global_function_declaration(name, function_expr, code, constants),
        Expression::Define { name: _, value: _ } => {
            todo!()
        }
        Expression::FunctionLiteral(function_expr) => {
            compile_function(None, function_expr);
        }
    }
}

// can this be implemented in terms of global declaration?
fn compile_global_function_declaration(
    name: String,
    function_expr: FunctionExpression,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
) {
    // potenially jank way of doing it for now:
    // make function object
    let function_obj = ConstantValue::Object(ConstantObject::Function(compile_function(
        Some(name.clone()),
        function_expr,
    )));
    // reference constant function object on stack
    code.push(Op::Constant.into());
    constants.push(function_obj);
    code.push(constants.len() as u8 - 1);
    // declare top of stack as global
    code.push(Op::DeclareGlobal.into());
    constants.push(ConstantValue::Object(ConstantObject::String(name)));
    code.push(constants.len() as u8 - 1);
}

fn compile_regular_form(
    exprs: Vec<Expression>,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
    locals: &mut Vec<String>,
) {
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

fn compile_global_declaration(
    value: Box<Expression>,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
    locals: &mut Vec<String>,
    name: String,
) {
    compile_expression(*value, code, constants, locals);
    code.push(Op::DeclareGlobal.into());
    constants.push(ConstantValue::Object(ConstantObject::String(name)));
    code.push(constants.len() as u8 - 1);
}

fn compile_if_statement(
    condition: Box<Expression>,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
    locals: &mut Vec<String>,
    else_: Box<Expression>,
    then: Box<Expression>,
) {
    // IF
    compile_expression(*condition, code, constants, locals);
    // skip to "then"
    code.push(Op::CondJump.into());
    code.push(0x00);
    // will mutate this later
    let then_jump_idx = code.len() - 1;
    // ELSE
    compile_expression(*else_, code, constants, locals);
    // skip to end
    code.push(Op::Jump.into());
    // code[to_then_jump_address as usize] = code.len() as u8;
    code.push(0x00);
    // will mutate this later
    let finish_jump_idx = code.len() - 1;
    // THEN
    let then_jump = (code.len() - then_jump_idx) as u8;
    code[then_jump_idx] = then_jump;
    compile_expression(*then, code, constants, locals);
    // FINISH
    let finish_jump = (code.len() - finish_jump_idx) as u8;
    code[finish_jump_idx] = finish_jump
}

fn compile_self_evaluation(
    sexpr: SrcSexpr,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
    quote_level: usize,
) {
    match sexpr {
        SrcSexpr::Bool(x) => {
            code.push(Op::Constant.into());
            constants.push(ConstantValue::Boolean(x));
            code.push(constants.len() as u8 - 1);
        }
        SrcSexpr::Int(x) => {
            code.push(Op::Constant.into());
            constants.push(ConstantValue::Integer(x));
            code.push(constants.len() as u8 - 1);
        }
        SrcSexpr::Float(x) => {
            code.push(Op::Constant.into());
            constants.push(ConstantValue::Float(x));
            code.push(constants.len() as u8 - 1);
        }
        SrcSexpr::String(x) => {
            code.push(Op::Constant.into());
            constants.push(ConstantValue::Object(ConstantObject::String(x)));
            code.push(constants.len() as u8 - 1);
        }
        SrcSexpr::Symbol(x) => {
            code.push(Op::Constant.into());
            constants.push(ConstantValue::Object(ConstantObject::Symbol(x)));
            code.push(constants.len() as u8 - 1);
        }
        // NOTE this is a literal sexpr list: `'()`, not a list constructor: `(list 1 2 3)`. The latter is a regular form
        SrcSexpr::List(sexprs) => {
            // nil for end of list
            code.push(Op::Constant.into());
            constants.push(ConstantValue::Nil);
            code.push(constants.len() as u8 - 1);

            // cons each element in reverse order
            for sexpr in sexprs.into_iter().rev() {
                compile_self_evaluation(sexpr, code, constants, quote_level);
                code.push(Op::Cons.into());
            }
        }
        SrcSexpr::Quote(quoted_sexpr) => {
            compile_self_evaluation(*quoted_sexpr, code, constants, quote_level + 1);
            // this is kinda hacky:
            // The first level of quoting is handled by the compiler, by compiling, for example, `'x` to the literal symbol `x`
            // in the case of `''x`, the first quote is handled by the compiler, and the second quote is handled here:
            if quote_level >= 2 {
                code.push(Op::Quote.into());
            }
        }
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

    // println!("{:#?}", expressions);

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
        panic!("Lexing error: {}", e);
    });
    // println!("{:#?}", tokens);

    let ast = parser::parse(tokens).unwrap_or_else(|e| {
        panic!("Parsing error: {}", e);
    });
    // println!("{:#?}", ast);

    compile_ast(ast)
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
        assert_eq!(vm.stack, StaticStack::from([SmallVal::Integer(12)]))
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
        assert_eq!(vm.stack, StaticStack::from([SmallVal::Integer(23)]))
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
        assert_eq!(vm.stack, StaticStack::from([SmallVal::Integer(205)]));
    }
    #[test]
    fn test_cons() {
        let bc = compile_expressions(vec![Expression::SrcSexpr(SrcSexpr::Quote(Box::new(
            SrcSexpr::List(vec![SrcSexpr::Int(1), SrcSexpr::Int(2), SrcSexpr::Int(3)]),
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
                SrcSexpr::Int(10),
                SrcSexpr::List(vec![SrcSexpr::Int(20), SrcSexpr::Int(30)]),
                SrcSexpr::Int(40),
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
