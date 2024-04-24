#![allow(unused, dead_code)]

use crate::sexpr::Sexpr;
use crate::static_stack::StaticStack;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::default;
use std::fmt::{format, Display};
use std::hash::Hash;
use std::ops::Deref;
use std::panic::PanicInfo;

// THOUGHTS
// First thought is that we may be able to mirror evaluator.~.eval with "produce bytecode that "
// simple stack-based virtual machine for integer arithmetic

const STACK_SIZE: usize = 4096;
pub struct VM {
    ip: *const u8,
    pub stack: StaticStack<SmallValue, STACK_SIZE>,
    pub globals: HashMap<String, SmallValue>,
    callframes: Vec<CallFrame>,
    heap: *mut HeapObject,
    current_chunk: BytecodeChunk,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectValue {
    SmallValue(SmallValue),
    String(String),
    Function(Function),
    Symbol(String),
    ConsCell(*mut ConsCell),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsCell(SmallValue, *mut ConsCell);

impl Display for ConsCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} . {})", self.0, &unsafe { &*self.1 })
    }
}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectValue::String(s) => write!(f, "{}", s),
            ObjectValue::Function(func) => write!(f, "function <{}>", func.name),
            ObjectValue::Symbol(s) => write!(f, ":{}", s),
            ObjectValue::ConsCell(cell) => write!(f, "{}", unsafe { &**cell }),
            ObjectValue::SmallValue(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    arity: usize,
    bytecode: Box<BytecodeChunk>,
}
impl Function {
    pub fn new(name: String, arity: usize, bytecode: BytecodeChunk) -> Self {
        Function {
            name,
            arity,
            bytecode: Box::new(bytecode),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeapObject {
    next: *mut HeapObject,
    value: ObjectValue,
    // marked: bool,
}

impl Display for HeapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmallValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
    Object(*mut HeapObject),
}

impl Display for SmallValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmallValue::Integer(i) => write!(f, "{}", i),
            SmallValue::Float(fl) => write!(f, "{}", fl),
            SmallValue::Boolean(b) => write!(f, "{}", b),
            SmallValue::Nil => write!(f, "nil"),
            SmallValue::Object(ptr) => write!(f, "{:?}", unsafe { &**ptr }),
        }
    }
}

impl default::Default for SmallValue {
    fn default() -> Self {
        SmallValue::Integer(69) // flag for debugging
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CallFrame {
    return_address: *const u8,
    stack_frame_start: usize,
    arity: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantsValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
    Object(ObjectValue),
}

impl SmallValue {
    fn truthy(&self) -> bool {
        match self {
            SmallValue::Integer(i) => *i != 0,
            SmallValue::Float(f) => *f != 0.0,
            SmallValue::Boolean(b) => *b,
            SmallValue::Nil => false,
            SmallValue::Object(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeChunk {
    pub code: Vec<u8>,
    pub constants: Vec<ConstantsValue>,
}

impl BytecodeChunk {
    pub fn new(code: Vec<u8>, constants: Vec<ConstantsValue>) -> Self {
        BytecodeChunk { code, constants }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, IntoPrimitive, TryFromPrimitive)]
pub enum Op {
    Constant = 0,
    Add = 1,
    Sub = 2,
    Mul = 3,
    Div = 4,
    Neg = 5,
    Jump = 6,     // jumps to the specified address
    CondJump = 7, // jumps to the specified address if the top of the stack is not zero
    FuncCall = 8,
    Return = 9,
    DeclareGlobal = 10,
    ReferenceGlobal = 11,
    ReferenceLocal = 12,

    // testing
    Cons = 13,

    DebugEnd = 254,   // ends the program
    DebugPrint = 255, // prints the stack
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> VM {
        let mut vm = VM {
            ip: std::ptr::null_mut(),
            stack: StaticStack::new(),
            heap: std::ptr::null_mut(),
            globals: HashMap::default(),
            callframes: Vec::default(),
            current_chunk: BytecodeChunk::new(vec![], vec![]),
        };

        let mul = unsafe {
            vm.allocate_value(ObjectValue::Function(Function {
                name: "*".to_string(),
                arity: 2,
                bytecode: Box::new(BytecodeChunk {
                    code: vec![
                        Op::ReferenceLocal.into(),
                        1,
                        Op::ReferenceLocal.into(),
                        2,
                        Op::Mul.into(),
                        Op::Return.into(),
                    ],
                    constants: vec![],
                }),
            }))
        };

        let add = unsafe {
            vm.allocate_value(ObjectValue::Function(Function {
                name: "+".to_string(),
                arity: 2,
                bytecode: Box::new(BytecodeChunk {
                    code: vec![
                        Op::ReferenceLocal.into(),
                        1,
                        Op::ReferenceLocal.into(),
                        2,
                        Op::Add.into(),
                        Op::Return.into(),
                    ],
                    constants: vec![],
                }),
            }))
        };

        vm.globals.insert("*".to_string(), SmallValue::Object(mul));
        vm.globals.insert("+".to_string(), SmallValue::Object(add));

        vm
    }

    pub fn run(&mut self, chunk: BytecodeChunk) {
        self.ip = chunk.code.as_ptr();
        self.current_chunk = chunk;
        loop {
            let byte: Op = unsafe { *self.ip }.try_into().unwrap();
            match byte {
                Op::Constant => self.handle_constant(),
                Op::CondJump => self.handle_cond_jump(),
                Op::Jump => self.handle_jump(),
                Op::Add => self.handle_add(),
                Op::Sub => self.handle_sub(),
                Op::Mul => self.handle_mul(),
                Op::Div => self.handle_div(),
                Op::Neg => self.handle_neg(),
                Op::DeclareGlobal => self.handle_declare_global(),
                Op::ReferenceGlobal => self.handle_reference_global(),
                Op::DebugPrint => self.handle_print(),
                Op::FuncCall => self.handle_func_call(),
                Op::Cons => self.handle_cons(),
                Op::ReferenceLocal => self.handle_reference_local(),
                Op::Return => self.handle_return(),
                Op::DebugEnd => return,
            }
        }
    }

    // the following are all in the wrong order and I don't care

    fn handle_cons(&mut self) {
        let mut car = self.stack.pop().unwrap();
        let mut cdr = self.stack.pop().unwrap();
        let heap_obj_ptr = match cdr {
            SmallValue::Object(o) => match unsafe { &*o }.value {
                ObjectValue::ConsCell(cdr_ptr) => unsafe { self.allocate_cons(car, cdr_ptr) },
                _ => panic!("expected cons cell"),
            },
            SmallValue::Nil => unsafe {
                self.allocate_cons(car, std::ptr::null_mut()) // this is potentially not quite right,
                                                              // I think we should maybe be allocating for SmallValue::Nil
            },
            _ => panic!("expected object or nil"),
        };
        self.stack.push(SmallValue::Object(heap_obj_ptr));
        self.advance();
    }

    unsafe fn allocate_cons(&mut self, car: SmallValue, cdr: *mut ConsCell) -> *mut HeapObject {
        let cons_ptr = alloc(Layout::new::<ConsCell>()) as *mut ConsCell;
        cons_ptr.write(ConsCell(car, cdr));
        self.allocate_value(ObjectValue::ConsCell(cons_ptr))
    }

    fn handle_return(&mut self) {
        let frame = self
            .callframes
            .pop()
            .expect("expected a call frame to return from");
        self.ip = frame.return_address;
        // clean up the stack
        let return_val = self.stack.pop().expect("expected a return value");
        // pop the arguments
        for _ in 0..frame.arity as usize {
            self.stack.pop();
        }
        // pop the function
        self.stack.pop();
        self.stack.push(return_val);
        self.advance();
    }

    fn handle_reference_local(&mut self) {
        let current_callframe = self
            .callframes
            .last()
            .expect("expected a call frame for a local variable");
        let stack_frame_start = current_callframe.stack_frame_start;
        let arity = current_callframe.arity;
        let offset = self.consume_next_byte_as_byte() as usize;
        let value = self.stack.at(stack_frame_start + offset).unwrap().clone();
        self.stack.push(value);
        self.advance();
    }

    fn handle_func_call(&mut self) {
        // expects the stack to be:
        // [..., function, arg1, arg2, ... argN]
        // and the operand to be the arity of the function, so we can lookup the function and args
        let given_arity = self.consume_next_byte_as_byte();

        let func_obj = match self.stack.peek_back(given_arity as usize).unwrap() {
            SmallValue::Object(obj) => match &unsafe { &*obj }.value {
                ObjectValue::Function(f) => f,
                _ => panic!("expected ObjectValue::Function"),
            },
            _ => panic!("expected StackValue::Object"),
        };

        if func_obj.arity != given_arity as usize {
            self.runtime_error(
                format!(
                    "arity mismatch: Expected {} arguments, got {}",
                    func_obj.arity, given_arity
                )
                .as_str(),
            )
        }

        self.callframes.push(self.make_callframe(func_obj));
        self.ip = func_obj.bytecode.code.as_ptr();
    }

    fn make_callframe(&self, func_obj: &Function) -> CallFrame {
        CallFrame {
            return_address: self.ip,
            stack_frame_start: {
                let ptr = self.stack.ptr - func_obj.arity as i32; // todo CHECK
                if ptr < 0 {
                    panic!();
                } else {
                    ptr as usize
                }
            },
            arity: func_obj.arity as usize,
        }
    }

    fn handle_print(&mut self) {
        let val = match self.stack.pop().unwrap() {
            SmallValue::Object(ptr) => format!("{}", &unsafe { &*ptr }),
            SmallValue::Integer(i) => i.to_string(),
            SmallValue::Float(f) => f.to_string(),
            SmallValue::Boolean(b) => b.to_string(),
            SmallValue::Nil => "nil".to_string(),
        };
        println!("{:?}", val);
        self.advance();
    }

    // fn handle_reference_global(&mut self, chunk: &BytecodeChunk) {
    fn handle_reference_global(&mut self) {
        let name = match self.consume_next_byte_as_constant() {
            SmallValue::Object(ptr) => match &unsafe { &*ptr }.value {
                ObjectValue::String(s) => s,
                _ => panic!("expected string value for reference"),
            },
            _ => panic!("expected string value for reference"),
        };
        let stack_val = self.globals.get(name).unwrap().clone();
        self.stack.push(stack_val);
        self.advance();
    }

    // fn handle_declare_global(&mut self, chunk: &BytecodeChunk) {
    fn handle_declare_global(&mut self) {
        let value = self.stack.pop().unwrap();
        let name = self.consume_next_byte_as_constant();
        match name {
            SmallValue::Object(ptr) => match &unsafe { &*ptr }.value {
                ObjectValue::String(s) => {
                    self.globals.insert(s.clone(), value);
                }
                _ => panic!("expected string"),
            },
            _ => panic!("expected string"),
        }
        self.advance();
    }

    fn handle_neg(&mut self) {
        let a = self.stack.pop().unwrap();
        self.stack.push(match a {
            SmallValue::Integer(a) => SmallValue::Integer(-a),
            _ => todo!(),
        });
        self.advance();
    }

    fn handle_div(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Integer(a / b),
            _ => todo!(),
        });
        self.advance();
    }

    fn handle_mul(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Integer(a * b),
            other => todo!("not implemented for {:?}", other),
        });
        self.advance();
    }

    fn handle_sub(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Integer(a - b),
            _ => todo!(),
        });
        self.advance();
    }

    fn handle_add(&mut self) {
        // reverse order because we pop from the stack
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        let result = match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Integer(a + b),
            (SmallValue::Object(a), SmallValue::Object(b)) => {
                match (&unsafe { &*a }.value, &unsafe { &*b }.value) {
                    (ObjectValue::String(a), ObjectValue::String(b)) => {
                        let obj_ptr = unsafe {
                            let obj_value = ObjectValue::String(a.clone() + b);
                            self.allocate_value(obj_value)
                        };
                        SmallValue::Object(obj_ptr)
                    }
                    _ => todo!(),
                }
            }
            otherwise => {
                print!("{:?}", otherwise);
                unimplemented!()
            }
        };
        self.stack.push(result);
        self.advance();
    }

    fn handle_jump(&mut self) {
        let offset = self.consume_next_byte_as_byte() as usize;
        self.ip = unsafe { self.ip.add(offset) };
    }

    fn handle_cond_jump(&mut self) {
        let mut offset = self.consume_next_byte_as_byte() as usize;
        let cond_val = self.stack.pop().unwrap();
        if !cond_val.truthy() {
            offset = 1;
        };
        self.ip = unsafe { self.ip.add(offset) };
        // println!("jumping {:?} forward", offset);
    }

    // fn handle_cond_jump(&mut self, chunk: &BytecodeChunk) {
    fn handle_constant(&mut self) {
        let constant = self.consume_next_byte_as_constant();
        // advances here
        self.stack.push(constant);
        self.advance();
    }

    // fn consume_next_byte_as_constant(&mut self, chunk: &BytecodeChunk) -> StackValue {
    fn consume_next_byte_as_constant(&mut self) -> SmallValue {
        unsafe {
            self.ip = self.ip.add(1);
            let constant_idx = *self.ip as usize;

            match self.current_chunk.constants[constant_idx].clone() {
                // IMPORTANT: clone
                ConstantsValue::Integer(v) => SmallValue::Integer(v),
                ConstantsValue::Float(v) => SmallValue::Float(v),
                ConstantsValue::Boolean(v) => SmallValue::Boolean(v),
                ConstantsValue::Nil => SmallValue::Nil,
                ConstantsValue::Object(value) => {
                    let obj_ptr = self.allocate_value(value);
                    SmallValue::Object(obj_ptr)
                }
            }
        }
    }

    fn consume_next_byte_as_byte(&mut self) -> u8 {
        unsafe {
            self.advance();
            *self.ip
        }
    }

    fn advance(&mut self) {
        unsafe {
            self.ip = self.ip.add(1);
        }
    }

    unsafe fn allocate_value(&mut self, obj_value: ObjectValue) -> *mut HeapObject {
        let obj_ptr = alloc(Layout::new::<HeapObject>()) as *mut HeapObject;
        obj_ptr.write(HeapObject {
            next: self.heap,
            value: obj_value,
        });
        self.heap = obj_ptr;
        obj_ptr
    }

    fn runtime_error(&self, message: &str) {
        panic!("Runtime error: {}", message);
        // std::process::exit(1);
    }

    // fn repl(&mut self) {
    //     loop {
    //         let mut input = String::new();
    //         std::io::stdin().read_line(&mut input).unwrap();
    //         let opcode: u8 = input.trim().parse().unwrap();
    //         let op = Op::try_from(opcode).unwrap();
    //         self.run(chunk)
    //     }
    // }

    // unsafe fn allocate<T>(obj: T) -> *mut T {
    //     let obj_ptr = alloc(Layout::new::<T>()) as *mut T;
    //     obj_ptr.write(obj);
    //     obj_ptr
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let mut vm = VM::default();
        let chunk = BytecodeChunk {
            code: vec![Op::Constant.into(), 0x00, Op::DebugEnd.into()],
            constants: vec![ConstantsValue::Integer(5)],
        };
        vm.run(chunk);
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(5)]));
    }

    #[test]
    fn test_simple_math() {
        let mut vm = VM::default();
        // push 5 push 6 add
        // 5 + 6 = 11
        let chunk = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0,
                Op::Constant.into(),
                1,
                Op::Add.into(),
                Op::DebugEnd.into(),
            ],
            constants: vec![ConstantsValue::Integer(5), ConstantsValue::Integer(6)],
        };
        vm.run(chunk);
        assert_eq!(vm.stack.peek_top().unwrap(), SmallValue::Integer(11))
    }

    #[test]
    fn test_cond() {
        let bytecode = vec![
            Op::Constant.into(),
            0,
            Op::CondJump.into(),
            5, // jump to the load
            Op::Constant.into(),
            1,
            Op::Jump.into(),
            3, // jump to the end
            Op::Constant.into(),
            2,
            Op::DebugEnd.into(),
        ];
        let ptr = bytecode.as_ptr();

        let mut vm = VM::default();
        vm.run(BytecodeChunk {
            code: bytecode,
            constants: vec![
                ConstantsValue::Integer(1),
                ConstantsValue::Integer(3),
                ConstantsValue::Integer(2),
            ],
        });
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(2)]));
        assert_eq!(vm.ip, unsafe { ptr.add(10) }); // idx after the last byte
    }

    #[test]
    fn test_cond_not() {
        let chunk = BytecodeChunk {
            code: vec![
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
            ],
            constants: vec![
                ConstantsValue::Integer(0),
                ConstantsValue::Integer(3),
                ConstantsValue::Integer(2),
            ],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        vm.run(chunk);
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(3)]));
        assert_eq!(vm.ip, unsafe { ptr.add(10) });
    }

    #[test]
    fn test_string() {
        let chunk = BytecodeChunk {
            code: vec![Op::Constant.into(), 0, Op::DebugEnd.into()],
            constants: vec![ConstantsValue::Object(ObjectValue::String(
                "Hello, world!".to_string(),
            ))],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 1);

        let string = match vm.stack.peek_top().unwrap() {
            SmallValue::Object(ptr) => match &unsafe { &*ptr }.value {
                ObjectValue::String(str) => str,
                _ => panic!(),
            },
            _ => panic!(),
        };

        assert_eq!(string, "Hello, world!");
        assert_eq!(vm.ip, unsafe { ptr.add(2) });
    }

    #[test]
    fn test_string_concat() {
        let chunk = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0,
                Op::Constant.into(),
                1,
                Op::Add.into(),
                Op::DebugEnd.into(),
            ],
            constants: vec![
                ConstantsValue::Object(ObjectValue::String("foo".to_string())),
                ConstantsValue::Object(ObjectValue::String("bar".to_string())),
            ],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 1);

        let string = match vm.stack.peek_top().unwrap() {
            SmallValue::Object(ptr) => match &unsafe { &*ptr }.value {
                ObjectValue::String(str) => str,
                _ => panic!(),
            },
            _ => panic!(),
        };

        assert_eq!(string, "foobar");
        assert_eq!(vm.ip, unsafe { ptr.add(5) });
    }

    #[test]
    fn test_var_declare() {
        let chunk = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0,
                Op::DeclareGlobal.into(),
                1,
                Op::DebugEnd.into(),
            ],
            constants: vec![
                ConstantsValue::Integer(5),                                     // value
                ConstantsValue::Object(ObjectValue::String("foo".to_string())), // name
            ],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        let num_globals_before = vm.globals.len();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 0);
        assert_eq!(vm.globals.len(), num_globals_before + 1);
        assert_eq!(vm.globals.get("foo").unwrap(), &SmallValue::Integer(5));
        assert_eq!(vm.ip, unsafe { ptr.add(4) });
    }

    #[test]
    fn test_var_reference() {
        let chunk = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0,
                Op::DeclareGlobal.into(),
                1,
                Op::ReferenceGlobal.into(),
                1,
                Op::DebugEnd.into(),
            ],
            constants: vec![
                ConstantsValue::Integer(5),                                     // value
                ConstantsValue::Object(ObjectValue::String("foo".to_string())), // name
            ],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.peek_top().unwrap(), SmallValue::Integer(5));
        assert_eq!(vm.ip, unsafe { ptr.add(6) });
    }

    #[test]
    fn test_function() {
        let bc = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0, // load the function
                Op::Constant.into(),
                1, // load the argument 20
                Op::Constant.into(),
                2, // load the argument 30
                Op::FuncCall.into(),
                2, // call the function with 2 arguments
                Op::DebugEnd.into(),
            ],
            constants: vec![
                ConstantsValue::Object(ObjectValue::Function(
                    Function {
                        name: "asdf".to_string(),
                        arity: 2,
                        bytecode: Box::new(BytecodeChunk {
                            code: vec![
                                Op::ReferenceLocal.into(),
                                // make variables 1-indexed as the function itself is at 0 (maybe? (bad idea? (probably)))
                                1, // load the first argument from back in the stack
                                Op::ReferenceLocal.into(),
                                2, // load the second argument from back in the stack
                                Op::Add.into(),
                                Op::Return.into(),
                            ],
                            constants: vec![],
                        }),
                    }, // )
                )),
                ConstantsValue::Integer(20),
                ConstantsValue::Integer(30),
            ],
        };

        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack.peek_top().unwrap(), SmallValue::Integer(50));
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(50)]));
    }

    #[test]
    fn test_advanced() {
        let bc = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0, // load the function
                Op::Constant.into(),
                1, // load the argument 20
                Op::Constant.into(),
                2, // load the argument 30
                Op::FuncCall.into(),
                2, // call the function with 2 arguments
                Op::DebugEnd.into(),
            ],
            constants: vec![
                ConstantsValue::Object(ObjectValue::Function(
                    Function {
                        name: "asdf".to_string(),
                        arity: 2,
                        bytecode: Box::new(BytecodeChunk {
                            code: vec![
                                Op::ReferenceLocal.into(),
                                // make variables 1-indexed as the function itself is at 0 (maybe? (bad idea? (probably)))
                                1, // load the first argument from back in the stack
                                Op::ReferenceLocal.into(),
                                2, // load the second argument from back in the stack
                                Op::Add.into(),
                                Op::Return.into(),
                            ],
                            constants: vec![],
                        }),
                    }, // )
                )),
                ConstantsValue::Integer(20),
                ConstantsValue::Integer(30),
            ],
        };

        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack.peek_top().unwrap(), SmallValue::Integer(50));
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(50)]));
    }

    // this is so jank but it'll do!
    #[test]
    fn test_cons() {
        let bc = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0, // nil
                Op::Constant.into(),
                1,               // 30
                Op::Cons.into(), // '(30 . nil)
                Op::DebugEnd.into(),
            ],
            constants: vec![ConstantsValue::Nil, ConstantsValue::Integer(30)],
        };

        let mut vm = VM::default();
        vm.run(bc);
        let cell = match vm.stack.peek_top().unwrap() {
            SmallValue::Object(v) => match &unsafe { &*v }.value {
                ObjectValue::ConsCell(cell) => *cell,
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_eq!(unsafe { &*cell }.0, SmallValue::Integer(30));
        assert_eq!(unsafe { &*cell }.1, std::ptr::null_mut());
    }

    // #[test]
    // fn test_constructing_cons_list() {
    //     let bc = BytecodeChunk {
    //         code: vec![Op::Constant.into(), 0, Op::Cons.into()],
    //         constants: vec![ConstantsValue::Integer(30)],
    //     };

    //     let mut vm = VM::default();
    //     vm.run(bc);
    //     match vm.stack.peek_top().unwrap() {
    //         SmallValue::Object(v) => match unsafe { &*v }.value {
    //             ObjectValue::ConsCell(cell) => {
    //                 assert_eq!(unsafe { (*cell) }.0, SmallValue::Integer(30));
    //                 assert_eq!(unsafe { (*cell) }.1, std::ptr::null_mut());
    //             }
    //             asdf => panic!("got {}", asdf),
    //         },
    //         _ => panic!(),
    //     };
    // }
}

// fn print_heap(head_: *mut HeapObject) {
//     unsafe {
//         println!(
//             "allocated {:?} (knowingly leaking memory for now)",
//             (*head_).clone()
//         );
//         println!("heap:");
//         let mut current = head_;
//         while !current.is_null() {
//             println!("- {:?}", &(*current).value);
//             current = (*current).next;
//         }
//     }
// }
