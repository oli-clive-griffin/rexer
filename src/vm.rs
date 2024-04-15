use num_enum::{IntoPrimitive, TryFromPrimitive};

/// THOUGHTS
/// First thought is that we may be able to mirror evaluator.~.eval with "produce bytecode that "

/// simple stack-based virtual machine for integer arithmetic
struct VM {
    ip: usize,
    code: Vec<u8>,
    stack: Vec<i32>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            ip: 0,
            code: vec![],
            stack: vec![],
        }
    }

    pub fn load(&mut self, code: Vec<u8>) {
        self.code = code;
    }

    pub fn run(&mut self) {
        loop {
            if self.ip >= self.code.len() {
                return;
            }

            let byte = self.code[self.ip];
            match byte {
                0x00 => {
                    let val = self.conume_byte();
                    self.stack.push(val);
                    self.ip += 1;
                }
                0x01 => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                    self.ip += 1;
                }
                0x02 => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a - b);
                    self.ip += 1;
                }
                0x03 => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a * b);
                    self.ip += 1;
                }
                0x04 => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a / b);
                    self.ip += 1;
                }
                0x05 => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(-a);
                    self.ip += 1;
                }
                _ => {
                    panic!("unknown opcode: {}", byte);
                }
            }
        }
    }

    fn conume_byte(&mut self) -> i32 {
        self.ip += 1;
        let byte = self.code[self.ip] as i32;
        byte
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, IntoPrimitive, TryFromPrimitive)]
enum Op {
    Load = 0x00,
    Add = 0x01,
    Sub = 0x02,
    Mul = 0x03,
    Div = 0x04,
    Neg = 0x05,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SimpleExpression {
    Call { op: Op, args: Vec<SimpleExpression> },
    Constant(u8),
}

fn compile_expression(expression: SimpleExpression, code: &mut Vec<u8>) {
    match expression {
        SimpleExpression::Call { op, args } => {
            for arg in args {
                compile_expression(arg, code);
            }
            code.push(op.into());
        }
        SimpleExpression::Constant(value) => {
            code.push(Op::Load.into());
            code.push(value)
        },
    }
}

fn compile(expression: SimpleExpression) -> Vec<u8> {
    let mut code: Vec<u8> = vec![];
    compile_expression(expression, &mut code);
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm() {
        let mut vm = VM::new();
        vm.load(vec![0x00, 0x05]);
        vm.run();
        println!("{:?}", vm.stack);
        assert_eq!(vm.stack[0], 5);
    }

    #[test]
    fn test_0() {
        let mut vm = VM::new();
        // push 5 push 6 add
        // 5 + 6 = 11
        vm.load(vec![0x00, 0x05, 0x00, 0x06, 0x01]);
        vm.run();
        assert_eq!(vm.stack[0], 11);
    }

    #[test]
    fn test_compile_0() {
        let expression = SimpleExpression::Call {
            op: Op::Add,
            args: vec![SimpleExpression::Constant(5), SimpleExpression::Constant(6)],
        };
        let code = compile(expression);
        assert_eq!(code, vec![0x00, 0x05, 0x00, 0x06, 0x01]);

        let mut vm = VM::new();
        vm.load(code);
        vm.run();
        println!("{:?}", vm.stack);
    }
}
