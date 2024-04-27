use rusp::compiler::compile;
use rusp::vm::VM;

#[test]
fn actually_e2e() {
    let src = r#"
(fn (a b) ((if b * +) 2 3))

(fn (c d e) (+ d e))

(* (a true) (c 2 3))
"#
    .to_owned();

    let bc = compile(&src);

    let mut vm = VM::default();
    vm.run(bc);
}

#[test]
fn actually_e2e_2() {
    let src = r#"
(fn (fib n)
    (if (< n 2)
        n
        (+ (fib (- n 1))
           (fib (- n 2)))))

(print (fib 20))
"#
    .to_owned();

    let bc = compile(&src);

    let fib = |n: i64| -> i64 {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..n {
            let c = a + b;
            a = b;
            b = c;
        }
        a
    };

    let mut vm = VM::default();
    vm.run(bc);
    println!("stack: {}", vm.stack);
    println!("globals: {:?}", vm.globals);
    assert_eq!(fib(20), *vm.stack.at(0).unwrap().as_integer().unwrap());
}
