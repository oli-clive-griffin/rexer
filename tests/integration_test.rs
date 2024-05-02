use rusp::compiler::compile;
use rusp::vm::VM;

#[test]
fn actually_e2e() {
    let src = r#"
(defun (a b) ((if b * +) 2 3))

(defun (_add d e) (+ d e))

(print (* (a true) ; 6
          (_add 2 3))) ; 5
"#
    .to_owned();

    let bc = compile(&src);

    let mut vm = VM::default();
    vm.run(bc);
}

#[test]
fn actually_e2e_2() {
    let src = r#"
(defun (fib n)
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
    assert_eq!(fib(20), *vm.stack.at(0).unwrap().as_integer().unwrap());
}

fn run_code(src: &str) {
    let bc = compile(&src.to_string());
    VM::default().run(bc);
}

#[test]
fn target_spec_1() {
    run_code("(print '\"asdf\")");
}

#[test]
fn target_spec_asdf() {
    run_code("(print '(1 '2))");
}

#[test]
fn target_spec_2() {
    run_code("(print 'asdf)");
}

#[test]
fn target_spec_3() {
    run_code("(print '(asdf))");
}

#[test]
fn target_spec_4() {
    run_code("(print '(asdf 1))");
}

#[test]
fn target_spec_5() {
    run_code("(print '(asdf (1)))");
}

#[test]
fn target_spec_6() {
    run_code("(print '(asdf '(1)))");
}

#[test]
fn target_spec_7() {
    run_code("(print ''1)");
}

#[test]
fn target_spec_8() {
    run_code("(print '\"a\")");
}

#[test]
fn target_spec_9() {
    run_code("(print (quote a))");
}

#[test]
fn target_spec_10() {
    run_code(r#"
(defun (f)
    (define z 10)
    (define y 10)
    (define x 10)
    (defun (g b)
        (+ b 1))
    g)
(print ((f) 8))
"#
    );
}

#[test]
fn target_spec_11() {
    run_code(r#"
(defun (f)
    (defun (g)
        (defun (h) "asdf")
        h)
    g)

(print (((f)) 8))
"#
    );
}

#[test]
fn target_spec_12() {
    run_code(
        r#"
(defun (foo arg1)
    (define local "baz")
    (print local)
    (print arg1))

(foo "bar")
"#,
    );
}

#[test]
fn target_spec() {
    let src = r#"
(fn (fib n)
    (if (< n 2)
        n
        (+ (fib (- n 1))
           (fib (- n 2)))))

(print (fib 20))

(define foo "bar")

(print foo)

; inner functions
(defun (fib-iter n)
    (defun (inner a b n)
        (if (= n 0)
            a
            (inner b (+ a b) (- n 1))))
    (inner 0 1 n))

(print (fib-iter 20))
(print "^ should be 6765")

; stateful functions + returning allocated values
(defun (stateful)
    (define x 0)
    (print (concat "returning " (stringify x)))
    (+ x 4))
(define y (stateful))
(print "^ should print 'returning 0'")
(print y)
(print "^ should be 4")


; closures
(defun (make-adder x)
    (fn (y) (+ x y)))
(define add10 (make-adder 10))
(print (add10 5))
(print "^ should be 15")

; stateful closures
(defun (counter)
    (define x 0)
    (fn ()
        (set! x (+ x 1))
        x)
(define c (counter))
(print (c))
(print "^ should be 1"
(print (c))
(print "^ should be 2"
(print (c))
(print "^ should be 3"

; higher order functions
(defun (apply-twice f x)
    (f (f x)))

(print (apply-twice (make-adder 10) 5))
(print "^ should be 25")

; cons cells
(define l '(1 2 3 4 5))
(print "^ should be (1 2 3 4 5)"

(print (car l))
(print "^ should be 1")

(print (cdr l))
(print "^ should be (2 3 4 5)"
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
    assert_eq!(fib(20), *vm.stack.at(0).unwrap().as_integer().unwrap());
}
