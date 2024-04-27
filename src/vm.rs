use crate::compiler::disassemble;
use crate::static_stack::StaticStack;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::default;
use std::fmt::{Debug, Display};

const STACK_SIZE: usize = 4096;
pub struct VM {
    ip: *const u8,
    pub stack: StaticStack<SmallValue, STACK_SIZE>,
    pub globals: HashMap<String, SmallValue>,
    callframes: Vec<CallFrame>,
    heap: *mut HeapObject,
    global_constants: Vec<ConstantValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectValue {
    SmallValue(SmallValue),
    String(String),
    Function(Function),
    Symbol(String),
    ConsCell(ConsCell),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsCell(SmallValue, *mut ConsCell);

impl Display for ConsCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let car = self.0;

        let cdr = if self.1.is_null() {
            "nil".to_string()
        } else {
            format!("{}", unsafe { &*self.1 })
        };

        write!(f, "({} . {})", car, cdr)
    }
}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectValue::String(s) => write!(f, "{}", s),
            ObjectValue::Function(func) => write!(f, "function <{}>", func.name),
            ObjectValue::Symbol(s) => write!(f, ":{}", s),
            ObjectValue::ConsCell(cell) => write!(f, "{}", cell),
            ObjectValue::SmallValue(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub bytecode: Box<BytecodeChunk>,
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function(name={}, arity={} bc={})",
            self.name,
            self.arity,
            indent(disassemble(&self.bytecode), 2)
        )
    }
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
    pub value: ObjectValue,
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
    ObjectPtr(*mut HeapObject),
}

impl Display for SmallValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmallValue::Integer(i) => write!(f, "{}", i),
            SmallValue::Float(fl) => write!(f, "{}", fl),
            SmallValue::Boolean(b) => write!(f, "{}", b),
            SmallValue::Nil => write!(f, "nil"),
            SmallValue::ObjectPtr(ptr) => write!(f, "{}", unsafe { &**ptr }),
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
    constants: Vec<ConstantValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
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
            SmallValue::ObjectPtr(_) => false,
        }
    }

    pub fn as_integer(&self) -> Option<&i64> {
        if let Self::Integer(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeChunk {
    pub code: Vec<u8>,
    pub constants: Vec<ConstantValue>,
}

impl BytecodeChunk {
    pub fn new(code: Vec<u8>, constants: Vec<ConstantValue>) -> Self {
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
    GT = 6,
    LT = 7,
    GTE = 8,
    LTE = 9,
    Jump = 10,     // jumps to the specified address
    CondJump = 11, // jumps to the specified address if the top of the stack is not zero
    FuncCall = 12,
    Return = 13,
    DeclareGlobal = 14,
    ReferenceGlobal = 15,
    ReferenceLocal = 16,
    Cons = 17, // really not sure this should be an opcode
    Print = 18,
    DebugEnd = 254, // ends the program
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

fn binary_function(name: &'static str, op: Op) -> Function {
    Function {
        name: name.to_string(),
        arity: 2,
        bytecode: Box::new(BytecodeChunk {
            code: vec![
                Op::ReferenceLocal.into(),
                1,
                Op::ReferenceLocal.into(),
                2,
                op.into(),
                Op::Return.into(),
            ],
            constants: vec![],
        }),
    }
}

fn builtins() -> Vec<Function> {
    vec![
        binary_function("*", Op::Mul),
        binary_function("+", Op::Add),
        binary_function("-", Op::Sub),
        binary_function("/", Op::Div),
        binary_function(">", Op::GT),
        binary_function("<", Op::LT),
        binary_function(">=", Op::GTE),
        binary_function("<=", Op::LTE),
        Function {
            name: "print".to_string(),
            arity: 1,
            bytecode: Box::new(BytecodeChunk {
                code: vec![
                    Op::ReferenceLocal.into(),
                    1,
                    Op::Print.into(),
                    Op::Return.into(),
                ],
                constants: vec![],
            }),
        },
    ]
}

impl VM {
    pub fn new() -> VM {
        let mut vm = VM {
            ip: std::ptr::null_mut(),
            stack: StaticStack::new(),
            heap: std::ptr::null_mut(),
            globals: HashMap::default(),
            callframes: Vec::default(),
            global_constants: Vec::default(),
        };

        for obj in builtins() {
            let name = obj.name.clone();
            let obj_ptr = unsafe { vm.allocate_value(ObjectValue::Function(obj)) };
            vm.globals.insert(name, SmallValue::ObjectPtr(obj_ptr));
        }

        vm
    }

    pub fn run(&mut self, chunk: BytecodeChunk) {
        self.ip = chunk.code.as_ptr();
        self.global_constants = chunk.constants;
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
                Op::GT => self.handle_gt(),
                Op::LT => self.handle_lt(),
                Op::GTE => self.handle_gte(),
                Op::LTE => self.handle_lte(),
                Op::DeclareGlobal => self.handle_declare_global(),
                Op::ReferenceGlobal => self.handle_reference_global(),
                Op::Print => self.handle_print(),
                Op::FuncCall => self.handle_func_call(),
                Op::Cons => self.handle_cons(),
                Op::ReferenceLocal => self.handle_reference_local(),
                Op::Return => self.handle_return(),
                Op::DebugEnd => return,
            }
        }
    }

    // the following are all in the wrong order oh well

    fn handle_cons(&mut self) {
        let car = self.stack.pop().unwrap();
        let cdr = self.stack.pop().unwrap();

        let heap_obj_ptr = match cdr {
            SmallValue::ObjectPtr(o) => unsafe {
                match &mut (*o).value {
                    ObjectValue::ConsCell(ref mut cdr_ptr) => self.allocate_value(
                        ObjectValue::ConsCell(ConsCell(car, cdr_ptr as *mut ConsCell)),
                    ),
                    _ => panic!("expected cons cell"),
                }
            },
            SmallValue::Nil => unsafe {
                self.allocate_value(ObjectValue::ConsCell(ConsCell(
                    car,
                    std::ptr::null_mut(), // This is potentially not quite right, I think we
                                          // should maybe be allocating for SmallValue::Nil
                )))
            },
            other => panic!("expected object or nil, got {other}"),
        };
        self.stack.push(SmallValue::ObjectPtr(heap_obj_ptr));
        self.advance();
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
        for _ in 0..frame.arity {
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
        let offset = self.consume_next_byte_as_byte() as usize;
        let value = *self.stack.at(stack_frame_start + offset).unwrap();
        self.stack.push(value);
        self.advance();
    }

    fn handle_func_call(&mut self) {
        // expects the stack to be:
        // [..., function, arg1, arg2, ... argN]
        // and the operand to be the arity of the function, so we can lookup the function and args
        let given_arity = self.consume_next_byte_as_byte();

        let func_obj = match self.stack.peek_back(given_arity as usize).unwrap() {
            SmallValue::ObjectPtr(obj) => match &unsafe { &*obj }.value {
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
            arity: func_obj.arity,
            constants: func_obj.bytecode.constants.clone(),
        }
    }

    fn handle_print(&mut self) {
        let val = self.stack.pop().unwrap();
        println!("{}", val);
        self.advance();
    }

    fn handle_reference_global(&mut self) {
        let name = match self.consume_next_byte_as_constant() {
            SmallValue::ObjectPtr(ptr) => match &unsafe { &*ptr }.value {
                ObjectValue::String(s) => s,
                got => panic!(
                    "expected ObjectPtr to be String for reference, got {:?}",
                    got
                ),
            },
            constant_val => panic!(
                "expected constant to be ObjectPtr(String) for reference, got constant {:?}",
                constant_val
            ),
        };
        let global = *self.globals.get(name).unwrap_or_else(|| {
            self.runtime_error(format!("undefined global variable: {}", name).as_str());
        });
        self.stack.push(global); // copy
        self.advance();
    }

    fn handle_declare_global(&mut self) {
        let value = self.stack.pop().unwrap();
        let name = self.consume_next_byte_as_constant();
        match name {
            SmallValue::ObjectPtr(ptr) => match &unsafe { &*ptr }.value {
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
        self.stack
            .push(SmallValue::Integer(-a.as_integer().unwrap()));
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

    fn handle_gt(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Boolean(a > b),
            _ => todo!(),
        });
        self.advance();
    }

    fn handle_lt(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Boolean(a < b),
            _ => todo!(),
        });
        self.advance();
    }

    fn handle_gte(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Boolean(a >= b),
            _ => todo!(),
        });
        self.advance();
    }

    fn handle_lte(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Boolean(a <= b),
            _ => todo!(),
        });
        self.advance();
    }

    fn handle_add(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        let result = match (a, b) {
            (SmallValue::Integer(a), SmallValue::Integer(b)) => SmallValue::Integer(a + b),
            (SmallValue::ObjectPtr(a), SmallValue::ObjectPtr(b)) => {
                match (&unsafe { &*a }.value, &unsafe { &*b }.value) {
                    (ObjectValue::String(a), ObjectValue::String(b)) => {
                        let obj_ptr = unsafe {
                            let obj_value = ObjectValue::String(a.clone() + b);
                            self.allocate_value(obj_value)
                        };
                        SmallValue::ObjectPtr(obj_ptr)
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
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
    }

    fn handle_constant(&mut self) {
        let constant = self.consume_next_byte_as_constant();
        self.stack.push(constant);
        self.advance();
    }

    fn consume_next_byte_as_constant(&mut self) -> SmallValue {
        unsafe {
            self.ip = self.ip.add(1);
            let constant_idx = *self.ip as usize;

            match self.get_constant(constant_idx) {
                // IMPORTANT: clone
                ConstantValue::Integer(i) => SmallValue::Integer(*i),
                ConstantValue::Float(f) => SmallValue::Float(*f),
                ConstantValue::Boolean(b) => SmallValue::Boolean(*b),
                ConstantValue::Nil => SmallValue::Nil,
                ConstantValue::Object(value) => {
                    let obj_ptr = self.allocate_value(value.clone());
                    SmallValue::ObjectPtr(obj_ptr)
                }
            }
        }
    }

    fn get_constant(&self, idx: usize) -> &ConstantValue {
        if let Some(frame) = &self.callframes.last() {
            return &frame.constants[idx];
        };
        &self.global_constants[idx]
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

    fn runtime_error(&self, message: &str) -> ! {
        panic!("Runtime error: {}", message);
    }
}

fn indent(s: String, level: usize) -> String {
    let indent = "  ".repeat(level);
    s.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let mut vm = VM::default();
        let chunk = BytecodeChunk {
            code: vec![Op::Constant.into(), 0x00, Op::DebugEnd.into()],
            constants: vec![ConstantValue::Integer(5)],
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
            constants: vec![ConstantValue::Integer(5), ConstantValue::Integer(6)],
        };
        vm.run(chunk);
        assert_eq!(vm.stack.peek_top().unwrap(), &SmallValue::Integer(11))
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
                ConstantValue::Integer(1),
                ConstantValue::Integer(3),
                ConstantValue::Integer(2),
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
                ConstantValue::Integer(0),
                ConstantValue::Integer(3),
                ConstantValue::Integer(2),
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
            constants: vec![ConstantValue::Object(ObjectValue::String(
                "Hello, world!".to_string(),
            ))],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 1);

        let string = match vm.stack.peek_top().unwrap() {
            SmallValue::ObjectPtr(ptr) => match &unsafe { &**ptr }.value {
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
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
                ConstantValue::Object(ObjectValue::String("bar".to_string())),
            ],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 1);

        let string = match vm.stack.peek_top().unwrap() {
            SmallValue::ObjectPtr(ptr) => match &unsafe { &**ptr }.value {
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
                ConstantValue::Integer(5),                                     // value
                ConstantValue::Object(ObjectValue::String("foo".to_string())), // name
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
                ConstantValue::Integer(5),                                     // value
                ConstantValue::Object(ObjectValue::String("foo".to_string())), // name
            ],
        };
        let ptr = chunk.code.as_ptr();

        let mut vm = VM::default();
        vm.run(chunk);
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.peek_top().unwrap(), &SmallValue::Integer(5));
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
                ConstantValue::Object(ObjectValue::Function(
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
                ConstantValue::Integer(20),
                ConstantValue::Integer(30),
            ],
        };

        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack.peek_top().unwrap(), &SmallValue::Integer(50));
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
                ConstantValue::Object(ObjectValue::Function(
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
                ConstantValue::Integer(20),
                ConstantValue::Integer(30),
            ],
        };

        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack.peek_top().unwrap(), &SmallValue::Integer(50));
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
            constants: vec![ConstantValue::Nil, ConstantValue::Integer(30)],
        };

        let mut vm = VM::default();
        vm.run(bc);
        let cell = match vm.stack.peek_top().unwrap() {
            SmallValue::ObjectPtr(v) => match &unsafe { &**v }.value {
                ObjectValue::ConsCell(cell) => cell,
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_eq!(cell.0, SmallValue::Integer(30));
        assert_eq!(cell.1, std::ptr::null_mut());
    }

    #[test]
    fn test_cons_2() {
        let bc = BytecodeChunk {
            code: vec![
                Op::Constant.into(),
                0, // nil
                Op::Constant.into(),
                1,               // 20
                Op::Cons.into(), // '(20 . nil)
                Op::Constant.into(),
                2,                   // 10
                Op::Cons.into(),     // '(10 . (20 . nil))
                Op::DebugEnd.into(), //
            ],
            constants: vec![
                ConstantValue::Nil,
                ConstantValue::Integer(20),
                ConstantValue::Integer(10),
            ],
        };

        let mut vm = VM::default();
        vm.run(bc);
        let cell = match *vm.stack.peek_top().unwrap() {
            SmallValue::ObjectPtr(v) => match &unsafe { &*v }.value {
                ObjectValue::ConsCell(cell) => cell,
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_eq!(&cell.0, &SmallValue::Integer(10));
    }

    impl<T: Default + Copy, const MAX: usize> StaticStack<T, MAX> {
        pub fn from<const N: usize>(values: [T; N]) -> Self {
            let mut stack = Self::new();
            for value in values {
                stack.push(value);
            }
            stack
        }

        pub fn peek_top(&self) -> Option<&T> {
            self.at(self.ptr as usize)
        }

        #[allow(clippy::len_without_is_empty)]
        pub fn len(&self) -> usize {
            (self.ptr + 1) as usize
        }
    }
}
