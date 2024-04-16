#![allow(unused, dead_code)]

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::alloc::{alloc, dealloc, Layout};

/// THOUGHTS
/// First thought is that we may be able to mirror evaluator.~.eval with "produce bytecode that "

/// simple stack-based virtual machine for integer arithmetic
pub struct VM {
    ip: *const u8,
    // code: Vec<u8>,
    // function_table: Vec<ByteCodeFunction>,
    pub stack: Vec<StackValue>, // todo remove pub
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectValue {
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeapObject {
    next: *mut HeapObject,
    value: ObjectValue,
    // marked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StackValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
    Object(*mut HeapObject),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantsValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
    Object(ObjectValue),
}

impl StackValue {
    fn truthy(&self) -> bool {
        match self {
            StackValue::Integer(i) => *i != 0,
            StackValue::Float(f) => *f != 0.0,
            StackValue::Boolean(b) => *b,
            StackValue::Nil => false,
            StackValue::Object(_) => false,
        }
    }
}

pub struct BytecodeChunk {
    pub code: Vec<u8>,
    pub constants: Vec<ConstantsValue>,
}
impl BytecodeChunk {
    pub fn new(code: Vec<u8>, constants: Vec<ConstantsValue>) -> Self {
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
    Load = 0,
    Add = 1,
    Sub = 2,
    Mul = 3,
    Div = 4,
    Neg = 5,
    Jump = 6,     // jumps to the specified address
    CondJump = 7, // jumps to the specified address if the top of the stack is not zero
    FuncCall = 8,
    DebugPrint = 255, // prints the stack
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
                    let mut offset = self.consume_next_byte_as_byte() as usize;
                    let cond_val = self.stack.pop().unwrap();

                    if !cond_val.truthy() {
                        offset = 1;
                    };

                    self.ip = unsafe { self.ip.add(offset) };
                }
                Op::Jump => {
                    let offset = self.consume_next_byte_as_byte() as usize;
                    self.ip = unsafe { self.ip.add(offset) };
                }
                Op::Add => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (StackValue::Integer(a), StackValue::Integer(b)) => {
                            StackValue::Integer(a + b)
                        }
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Sub => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (StackValue::Integer(a), StackValue::Integer(b)) => {
                            StackValue::Integer(a - b)
                        }
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Mul => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (StackValue::Integer(a), StackValue::Integer(b)) => {
                            StackValue::Integer(a * b)
                        }
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Div => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (StackValue::Integer(a), StackValue::Integer(b)) => {
                            StackValue::Integer(a / b)
                        }
                        _ => todo!(),
                    });
                    self.advance();
                }
                Op::Neg => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(match a {
                        StackValue::Integer(a) => StackValue::Integer(-a),
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
                Op::DebugPrint => {
                    println!("{:?}", self.stack);
                    self.advance();
                }
                _ => todo!(),
            }
        }
    }

    fn consume_next_byte_as_constant(&mut self, chunk: &BytecodeChunk) -> StackValue {
        unsafe {
            self.ip = self.ip.add(1);
            match chunk.constants[*self.ip as usize].clone() {
                ConstantsValue::Integer(v) => StackValue::Integer(v),
                ConstantsValue::Float(v) => StackValue::Float(v),
                ConstantsValue::Boolean(v) => StackValue::Boolean(v),
                ConstantsValue::Nil => StackValue::Nil,
                ConstantsValue::Object(value) => {
                    // let obj_ptr = allocate(HeapObject {
                    //     next: std::ptr::null_mut(),
                    //     value,
                    // });
                    let obj_ptr = allocate_value(value);
                    StackValue::Object(obj_ptr)
                }
            }
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

unsafe fn allocate<T>(obj: T) -> *mut T {
    let obj_ptr = alloc(Layout::new::<T>()) as *mut T;
    obj_ptr.write(obj);
    obj_ptr
}

unsafe fn allocate_value(obj_value: ObjectValue) -> *mut HeapObject {
    let obj_ptr = alloc(Layout::new::<HeapObject>()) as *mut HeapObject;
    obj_ptr.write(HeapObject {
        next: std::ptr::null_mut(),
        value: obj_value,
    });
    println!("LEAKING MEMORY allocated {:?}", obj_ptr);
    obj_ptr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let mut vm = VM::new();
        let chunk = BytecodeChunk {
            code: vec![0x00, 0x00],
            constants: vec![ConstantsValue::Integer(5)],
        };
        vm.run(chunk);
        assert_eq!(vm.stack, vec![StackValue::Integer(5)])
    }

    #[test]
    fn test_simple_math() {
        let mut vm = VM::new();
        // push 5 push 6 add
        // 5 + 6 = 11
        let chunk = BytecodeChunk {
            code: vec![Op::Load.into(), 0, Op::Load.into(), 1, Op::Add.into()],
            constants: vec![ConstantsValue::Integer(5), ConstantsValue::Integer(6)],
        };
        vm.run(chunk);
        assert_eq!(vm.stack[0], StackValue::Integer(11))
    }

    #[test]
    fn test_cond() {
        let bytecode = vec![
            Op::Load.into(),
            0,
            Op::CondJump.into(),
            5, // jump to the load
            Op::Load.into(),
            1,
            Op::Jump.into(),
            3, // jump to the end
            Op::Load.into(),
            2,
        ];
        let ptr = bytecode.as_ptr();

        let mut vm = VM::new();
        vm.run(BytecodeChunk {
            code: bytecode,
            constants: vec![
                ConstantsValue::Integer(1),
                ConstantsValue::Integer(3),
                ConstantsValue::Integer(2),
            ],
        });
        assert_eq!(vm.stack, vec![StackValue::Integer(2)]);
        assert_eq!(vm.ip, unsafe { ptr.add(10) }); // idx after the last byte
    }

    #[test]
    fn test_cond_not() {
        let chunk = BytecodeChunk {
            code: vec![
                Op::Load.into(),
                0,
                Op::CondJump.into(),
                5,
                Op::Load.into(),
                1,
                Op::Jump.into(),
                3,
                Op::Load.into(),
                2,
            ],
            constants: vec![
                ConstantsValue::Integer(0),
                ConstantsValue::Integer(3),
                ConstantsValue::Integer(2),
            ],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::new();
        vm.run(chunk);
        assert_eq!(vm.stack, vec![StackValue::Integer(3)]);
        assert_eq!(vm.ip, unsafe { ptr.add(10) });
    }

    #[test]
    fn test_string() {
        let chunk = BytecodeChunk {
            code: vec![Op::Load.into(), 0],
            constants: vec![ConstantsValue::Object(ObjectValue::String(
                "Hello, world!".to_string(),
            ))],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::new();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 1);

        let string = match vm.stack[0] {
            StackValue::Object(ptr) => match &unsafe { &*ptr }.value {
                ObjectValue::String(str) => str,
            }
            _ => panic!(),
        };

        assert_eq!(string, "Hello, world!");
        assert_eq!(vm.ip, unsafe { ptr.add(2) });
    }
}

// not needed for now
// // #[repr(C)] // for the struct definition
// impl StackValue {
//     fn from_bytes(bytes: [u8; 16]) -> Self {
//         unsafe { mem::transmute(bytes) }
//     }
//     fn to_bytes(self) -> [u8; 16] {
//         unsafe { mem::transmute(self) }
//     }
// }
