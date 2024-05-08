use crate::vm::{ConsCell, ObjectValue, SmallVal, VM};

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltIn {
    pub name: &'static str,
    pub arity: usize,
    pub func: fn(Vec<SmallVal>, &mut VM) -> SmallVal, // This signature is probably wrong, if we want cons to be able to return a &mut SmallVal for example
}

const ADD: BuiltIn = BuiltIn {
    name: "+",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Integer(i + j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Float(i + j),
            _ => panic!("add must be called with two integers"),
        }
    },
};

const SUB: BuiltIn = BuiltIn {
    name: "-",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Integer(i - j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Float(i - j),
            _ => panic!("sub must be called with two integers"),
        }
    },
};

const MUL: BuiltIn = BuiltIn {
    name: "*",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Integer(i * j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Float(i * j),
            _ => panic!("mul must be called with two integers"),
        }
    },
};

const DIV: BuiltIn = BuiltIn {
    name: "/",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Integer(i / j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Float(i / j),
            _ => panic!("div must be called with two integers"),
        }
    },
};

const MOD: BuiltIn = BuiltIn {
    name: "%",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Integer(i % j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Float(i % j),
            _ => panic!("mod must be called with two integers"),
        }
    },
};

const INC: BuiltIn = BuiltIn {
    name: "inc",
    arity: 1,
    func: |args, _vm| match &args[0] {
        SmallVal::Integer(i) => SmallVal::Integer(i + 1),
        got => panic!("must be called with an integer, got {:?}, {}", got, got),
    },
};

const PRINT: BuiltIn = BuiltIn {
    name: "print",
    arity: 1,
    func: |args, _vm| {
        println!("{}", args[0]);
        SmallVal::Nil
    },
};

const EQ: BuiltIn = BuiltIn {
    name: "=",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Bool(i == j),
            [SmallVal::Float(i), SmallVal::Float(j)] => SmallVal::Bool(i == j),
            [SmallVal::ObjectPtr(a), SmallVal::ObjectPtr(b)] => SmallVal::Bool(a == b), // todo watch out for this
            _ => panic!(),
        }
    },
};

const GT: BuiltIn = BuiltIn {
    name: ">",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Bool(i > j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => panic!(),
        }
    },
};

const LT: BuiltIn = BuiltIn {
    name: "<",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Bool(i < j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => panic!(),
        }
    },
};

const GTE: BuiltIn = BuiltIn {
    name: ">=",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Bool(i >= j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => panic!(),
        }
    },
};

const LTE: BuiltIn = BuiltIn {
    name: "<=",
    arity: 2,
    func: |args, _vm| {
        match args[..] {
            [SmallVal::Integer(i), SmallVal::Integer(j)] => SmallVal::Bool(i <= j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => panic!(),
        }
    },
};

const AND: BuiltIn = BuiltIn {
    name: "and",
    arity: 2,
    func: |args, _vm| match args[..] {
        [SmallVal::Bool(i), SmallVal::Bool(j)] => SmallVal::Bool(i && j),
        _ => panic!(),
    },
};

const OR: BuiltIn = BuiltIn {
    name: "or",
    arity: 2,
    func: |args, _vm| match args[..] {
        [SmallVal::Bool(i), SmallVal::Bool(j)] => SmallVal::Bool(i || j),
        _ => panic!(),
    },
};

const NOT: BuiltIn = BuiltIn {
    name: "not",
    arity: 1,
    func: |args, _vm| match args[0] {
        SmallVal::Bool(i) => SmallVal::Bool(!i),
        _ => panic!(),
    },
};

const CAR: BuiltIn = BuiltIn {
    name: "car",
    arity: 1,
    func: |args, _vm| match args[0] {
        SmallVal::ObjectPtr(ptr) => match &unsafe { &*ptr }.value {
            &ObjectValue::ConsCell(ConsCell(val_ptr, _cdr_ptr)) => SmallVal::ObjectPtr(val_ptr),
            _ => panic!("car must be called with a cons cell"),
        },
        _ => panic!(),
    },
};

const CDR: BuiltIn = BuiltIn {
    name: "cdr",
    arity: 1,
    func: |args, _vm| match args[0] {
        SmallVal::ObjectPtr(ptr) => match unsafe { &*ptr }.value {
            ObjectValue::ConsCell(ConsCell(_val_ptr, cdr_ptr)) => SmallVal::ObjectPtr(cdr_ptr),
            _ => panic!("car must be called with a cons cell"),
        },
        _ => panic!(),
    },
};

const CONS: BuiltIn = BuiltIn {
    name: "cons",
    arity: 2,
    func: |args, vm| {
        let car_val = args[0].clone();
        let cdr_val = args[1].clone();

        let car = unsafe {
            match car_val {
                SmallVal::Integer(_) | SmallVal::Float(_) | SmallVal::Bool(_) | SmallVal::Nil => {
                    vm.allocate_value(ObjectValue::SmallValue(car_val))
                }
                SmallVal::ObjectPtr(ptr) | SmallVal::Quote(ptr) => ptr,
            }
        };

        let cdr = unsafe {
            match cdr_val {
                SmallVal::Integer(_) | SmallVal::Float(_) | SmallVal::Bool(_) | SmallVal::Nil => {
                    vm.allocate_value(ObjectValue::SmallValue(cdr_val))
                }
                SmallVal::ObjectPtr(ptr) | SmallVal::Quote(ptr) => ptr,
            }
        };

        let cons_ptr = unsafe { vm.allocate_value(ObjectValue::ConsCell(ConsCell(car, cdr))) };

        SmallVal::ObjectPtr(cons_ptr)
    },
};

pub const BUILT_INS: [&BuiltIn; 18] = [
    &ADD, &SUB, &MUL, &DIV, &MOD, &INC, &PRINT, &EQ, &GT, &LT, &GTE, &LTE, &AND, &OR, &NOT, &CAR,
    &CDR, &CONS,
];
