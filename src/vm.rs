// #![allow(unused_variables, unused_imports, dead_code, unused_mut, unused_assignments, unused_unsafe, unused_must_use, unused_parens, unused_import_braces, private_interfaces)]

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// THOUGHTS
/// First thought is that we may be able to mirror evaluator.~.eval with "produce bytecode that "

/// simple stack-based virtual machine for integer arithmetic
pub struct VM {
    ip: *const u8,
    // code: Vec<u8>,
    pub stack: Vec<Value>, // todo remove pub
                           // function_table: Vec<ByteCodeFunction>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Boolean(b) => *b,
            Value::Nil => false,
        }
    }

}

pub struct BytecodeChunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}
impl BytecodeChunk {
    pub fn new(code: Vec<u8>, constants: Vec<Value>) -> Self {
        BytecodeChunk { code, constants }
    }
}


// struct ByteCodeFunction {
//     _name: String,
//     arity: usize,
//     bytecode: Vec<u8>,
// }

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, IntoPrimitive, TryFromPrimitive)]
pub enum Op {
    Load = 0x00,
    Add = 0x01,
    Sub = 0x02,
    Mul = 0x03,
    Div = 0x04,
    Neg = 0x05,
    Jump = 0x06,     // jumps to the specified address
    CondJump = 0x07, // jumps to the specified address if the top of the stack is not zero
    FuncCall = 0x08,
}

impl VM {
    pub fn new() -> VM {
        VM {
            ip: std::ptr::null_mut(),
            // current_chunk
            // code: vec![],
            stack: vec![],
            // function_table: vec![],
        }
    }

    // pub fn load(&mut self, code: Vec<u8>) {
    //     self.code = code;
    // }

    // pub fn run(&mut self) {
    pub fn run(&mut self, chunk: BytecodeChunk) {
        self.ip = chunk.code.as_ptr();
        let end_ptr = unsafe { self.ip.add(chunk.code.len()) };

        loop {
            if self.ip >= end_ptr {
                return;
            }

            // probably should switch back to raw bytes
            // but this is nice for development
            let byte: Op = unsafe { *self.ip }.try_into().unwrap();

            match byte {
                Op::Load => {
                    let constant = self.consume_next_byte_as_constant(&chunk); // advances here
                    self.stack.push(constant);
                    self.advance();
                }
                Op::CondJump => {
                    // get the jump offset
                    let mut offset = self.consume_next_byte_as_byte() as usize;
                    let cond_val = self.stack.pop().unwrap();

                    if !cond_val.truthy() {
                        offset = 1;
                    };

                    // println!("offset: {offset}");
                    // println!("befor ip: {:?}", ip);
                    self.ip = unsafe { self.ip.add(offset) };
                    // println!("after ip: {:?}", ip);

                }
                Op::Jump => {
                    let offset = self.consume_next_byte_as_byte() as usize;
                    // println!("offset: {offset}");
                    // println!("befor ip: {:?}", ip);
                    self.ip = unsafe { self.ip.add(offset) };
                    // println!("after ip: {:?}", ip);
                }
                Op::Add => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a + b),
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Sub => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Mul => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a * b),
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Div => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a / b),
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Neg => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(match a {
                        Value::Integer(a) => Value::Integer(-a),
                        _ => todo!(),
                    });
                    self.advance();
                }
                
                // Op::FuncCall => todo!(),
                // Op::FuncCall => {
                //     // expects the stack to be (top-down):
                //     // [<func_idx>, ...func_args,
                //     let func_idx = self.stack.pop().unwrap();
                //     let ByteCodeFunction {
                //         _name: _,
                //         bytecode,
                //         arity,
                //     } = &self.function_table[func_idx as usize];
                // }
                _ => todo!(),
            }
        }
    }

    fn consume_next_byte_as_constant(&mut self, chunk: &BytecodeChunk) -> Value {
        unsafe {
            self.ip = self.ip.add(1);
            chunk.constants[*self.ip as usize]
        }
    }

    fn consume_next_byte_as_byte(&mut self) -> u8 {
        unsafe {
            self.ip = self.ip.add(1);
            *self.ip
        }
    }

    fn advance(&mut self) {
        unsafe {
            self.ip = self.ip.add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let mut vm = VM::new();
        let chunk = BytecodeChunk {
            code: vec![0x00, 0x00],
            constants: vec![Value::Integer(5)],
        };
        vm.run(chunk);
        assert_eq!(vm.stack, vec![Value::Integer(5)])
    }

    #[test]
    fn test_simple_math() {
        let mut vm = VM::new();
        // push 5 push 6 add
        // 5 + 6 = 11
        let chunk = BytecodeChunk {
            code: vec![
                Op::Load.into(), 0,
                Op::Load.into(), 1,
                Op::Add.into(),
            ],
            constants: vec![Value::Integer(5), Value::Integer(6)],
        };
        vm.run(chunk);
        assert_eq!(vm.stack[0], Value::Integer(11))
    }

    #[test]
    fn test_cond() {
        let bytecode = vec![
            Op::Load.into(), 0,
            Op::CondJump.into(), 5, // jump to the load
            Op::Load.into(), 1,
            Op::Jump.into(), 3, // jump to the end
            Op::Load.into(), 2,
        ];
        let ptr = bytecode.as_ptr();

        let mut vm = VM::new();
        vm.run(BytecodeChunk {
            code: bytecode,
            constants: vec![Value::Integer(1), Value::Integer(3), Value::Integer(2)],
        });
        assert_eq!(vm.stack, vec![Value::Integer(2)]);
        assert_eq!(vm.ip, unsafe { ptr.add(10) }); // idx after the last byte
    }

    #[test]
    fn test_cond_not() {
        let chunk = BytecodeChunk {
            code: vec![
                Op::Load.into(), 0,
                Op::CondJump.into(), 5,
                Op::Load.into(), 1,
                Op::Jump.into(), 3,
                Op::Load.into(), 2,
            ],
            constants: vec![Value::Integer(0), Value::Integer(3), Value::Integer(2)],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::new();
        vm.run(chunk);
        assert_eq!(vm.stack, vec![Value::Integer(3)]);
        assert_eq!(vm.ip, unsafe { ptr.add(10) });
    }
}


// not needed for now
// // #[repr(C)] // for the struct definition
// impl Value {
//     fn from_bytes(bytes: [u8; 16]) -> Self {
//         unsafe { mem::transmute(bytes) }
//     }
//     fn to_bytes(self) -> [u8; 16] {
//         unsafe { mem::transmute(self) }
//     }
// }