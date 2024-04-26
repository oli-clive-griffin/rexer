use rusp::{
    compiler::{compile_expressions, compile_sexprs, disassemble, SimpleExpression},
    sexpr::Sexpr,
    vm::{ConstantValue, ObjectValue, SmallValue, VM},
};

#[test]
fn e2e_1() {
    let program = vec![
        SimpleExpression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(SimpleExpression::RegularForm(vec![
                SimpleExpression::Symbol("+".to_string()),
                SimpleExpression::Constant(ConstantValue::Integer(11)),
                SimpleExpression::Constant(ConstantValue::Integer(12)),
            ])),
        },
        SimpleExpression::DebugPrint(Box::new(SimpleExpression::If {
            condition: Box::new(SimpleExpression::Symbol("foo".to_string())),
            then: Box::new(SimpleExpression::Constant(ConstantValue::Object(
                ObjectValue::String("true".to_string()),
            ))),
            else_: Box::new(SimpleExpression::Constant(ConstantValue::Object(
                ObjectValue::String("false".to_string()),
            ))),
        })),
    ];

    let bc = compile_expressions(program);

    let mut vm = VM::default();
    vm.run(bc);
}

#[test]
fn e2e_2() {
    let program = vec![SimpleExpression::RegularForm(vec![
        SimpleExpression::If {
            condition: Box::new(SimpleExpression::Constant(ConstantValue::Boolean(true))),
            then: Box::new(SimpleExpression::Symbol("*".to_string())),
            else_: Box::new(SimpleExpression::Symbol("+".to_string())),
        },
        SimpleExpression::Constant(ConstantValue::Integer(2)),
        SimpleExpression::Constant(ConstantValue::Integer(3)),
    ])];

    let bc = compile_expressions(program);

    let mut vm = VM::default();

    vm.run(bc);
    println!("\n\nTEST:");
    println!("stack: {}", vm.stack);
    println!("globals: {:?}", vm.globals);
}

#[test]
#[should_panic(expected = "Runtime error: arity mismatch: Expected 2 arguments, got 3")]
fn e2e_3() {
    let program = vec![SimpleExpression::RegularForm(vec![
        SimpleExpression::Symbol("*".to_string()),
        SimpleExpression::Constant(ConstantValue::Integer(2)),
        SimpleExpression::Constant(ConstantValue::Integer(3)),
        SimpleExpression::Constant(ConstantValue::Integer(4)),
    ])];

    let bc = compile_expressions(program);

    let mut vm = VM::default();

    vm.run(bc);
}

#[test]
fn e2e_4() {
    let bc = compile_expressions(vec![SimpleExpression::Quote(Sexpr::List {
        quasiquote: false,
        sexprs: vec![
            Sexpr::Int(10),
            Sexpr::List {
                quasiquote: false,
                sexprs: vec![Sexpr::Int(20), Sexpr::Int(30)],
            },
            Sexpr::Int(40),
        ],
    })]);

    let mut vm = VM::default();
    vm.run(bc);
    let list = *vm.stack.at(0).unwrap();
    match list {
        SmallValue::ObjectPtr(o) => println!("{}", unsafe { &*o }.value),
        _ => panic!("Expected list"),
    }
}

#[test]
fn actually_e2e() {
    let src = r#"
(fn (a b) ((if b * +) 2 3))

(fn (c d e) (+ d e))

(define foo 10)

(* (a true) (c 2 3)
"#
    .to_owned();

    let tokens = rusp::lexer::lex(&src).unwrap();
    let ast = rusp::parser::parse(tokens).unwrap();
    let bc = compile_sexprs(ast.expressions);

    let mut vm = VM::default();
    vm.run(bc);
    println!("\n\nTEST:");
    println!("stack: {}", vm.stack);
    println!("globals: {:?}", vm.globals);
}
