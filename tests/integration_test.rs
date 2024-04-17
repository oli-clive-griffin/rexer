use risp::{
    compiler::{compile_program, SimpleExpression},
    vm::{ConstantsValue, ObjectValue, Op, VM},
};

#[test]
fn compiler() {
    let program = vec![
        SimpleExpression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(SimpleExpression::Call {
                op: Op::Add,
                args: Box::new((
                    SimpleExpression::Constant(ConstantsValue::Integer(11)),
                    SimpleExpression::Constant(ConstantsValue::Integer(12)),
                )),
            }),
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

    // NOTE: This test shouldn't be here but good for easy testing
    let mut vm = VM::default();
    vm.run(bc);
    println!("\n\n");
    println!("{:?}", vm.stack);
    println!("{:?}", vm.globals);
}
