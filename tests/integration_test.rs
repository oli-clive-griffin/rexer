use rusp::{
    compiler::{compile_program, SimpleExpression},
    vm::{ConstantsValue, ObjectValue, VM},
};

#[test]
fn e2e_1() {
    let program = vec![
        SimpleExpression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(SimpleExpression::Add(Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(11)),
                SimpleExpression::Constant(ConstantsValue::Integer(12)),
            )))),
        },
        SimpleExpression::DebugPrint(Box::new(SimpleExpression::If {
            condition: Box::new(SimpleExpression::Symbol("foo".to_string())),
            then: Box::new(SimpleExpression::Constant(ConstantsValue::Object(
                ObjectValue::String("true".to_string()),
            ))),
            else_: Box::new(SimpleExpression::Constant(ConstantsValue::Object(
                ObjectValue::String("false".to_string()),
            ))),
        })),
    ];

    let bc = compile_program(program);

    let mut vm = VM::default();
    vm.run(bc);
    println!("\n\nTEST:");
    println!("stack: {}", vm.stack);
    println!("globals: {:?}", vm.globals);
}

#[test]
fn e2e_2() {
    let program = vec![SimpleExpression::RegularForm(vec![
        SimpleExpression::If {
            condition: Box::new(SimpleExpression::Constant(ConstantsValue::Boolean(true))),
            then: Box::new(SimpleExpression::Symbol("*".to_string())),
            else_: Box::new(SimpleExpression::Symbol("+".to_string())),
        },
        SimpleExpression::Constant(ConstantsValue::Integer(2)),
        SimpleExpression::Constant(ConstantsValue::Integer(3)),
    ])];

    let bc = compile_program(program);

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
        SimpleExpression::Constant(ConstantsValue::Integer(2)),
        SimpleExpression::Constant(ConstantsValue::Integer(3)),
        SimpleExpression::Constant(ConstantsValue::Integer(4)),
    ])];

    let bc = compile_program(program);

    let mut vm = VM::default();

    vm.run(bc);
}
