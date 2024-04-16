use num_enum::{IntoPrimitive, TryFromPrimitive};

/// THOUGHTS
/// First thought is that we may be able to mirror evaluator.~.eval with "produce bytecode that "

/// simple stack-based virtual machine for integer arithmetic
pub struct VM {
    ip: usize,
    code: Vec<u8>,
    pub stack: Vec<i32>, // todo remove pub
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, IntoPrimitive, TryFromPrimitive)]
pub enum Op {
    Load = 0x00,
    Add = 0x01,
    Sub = 0x02,
    Mul = 0x03,
    Div = 0x04,
    Neg = 0x05,
    Jump = 0x06, // jumps to the specified address
    CondJump = 0x07, // jumps to the specified address if the top of the stack is not zero
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

            // probably should switch back to raw bytes
            // but this is nice for development
            let byte: Op  = self.code[self.ip].try_into().unwrap();
            match byte {
                Op::Load => {
                    let val = self.consume_byte();
                    self.stack.push(val as i32);
                    self.ip += 1;
                }
                Op::Add => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                    self.ip += 1;
                }
                Op::Sub => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a - b);
                    self.ip += 1;
                }
                Op::Mul => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a * b);
                    self.ip += 1;
                }
                Op::Div => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a / b);
                    self.ip += 1;
                }
                Op::Neg => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(-a);
                    self.ip += 1;
                }
                Op::CondJump => {
                    let condition_val = self.stack.pop().unwrap();
                    let addr = self.consume_byte();
                    if condition_val != 0 {
                        self.ip = addr as usize;
                    } else {
                        self.ip += 1;
                    }
                }
                Op::Jump => {
                    let addr = self.consume_byte();    
                    self.ip = addr as usize;
                }
            }
        }
    }

    fn consume_byte(&mut self) -> u8 {
        self.ip += 1;
        let byte = self.code[self.ip];
        byte
    }

    fn current_byte(&mut self) -> u8 {
        let byte = self.code[self.ip];
        byte
    }
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
    fn test_cond() {
        let bytecode = vec![
            Op::Load.into(),
            1,
            Op::CondJump.into(),
            8,
            Op::Load.into(),
            3,
            Op::Jump.into(),
            10,
            Op::Load.into(),
            2,
        ];

        let mut vm = VM::new();
        vm.load(bytecode);
        vm.run();
        assert_eq!(vm.stack, vec![2]);
    }

    #[test]
    fn test_cond_not() {
        let bytecode = vec![
            Op::Load.into(),
            0,
            Op::CondJump.into(),
            8,
            Op::Load.into(),
            3,
            Op::Jump.into(),
            10,
            Op::Load.into(),
            2,
        ];

        let mut vm = VM::new();
        vm.load(bytecode);
        vm.run();
        assert_eq!(vm.stack, vec![3]);
    }
}


