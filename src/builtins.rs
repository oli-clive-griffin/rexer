use std::str;

use crate::runtime_value::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RustFunc(fn(&[RuntimeValue]) -> RuntimeValue);

impl RustFunc {
    pub fn eval(&self, args: &[RuntimeValue]) -> RuntimeValue {
        (self.0)(args)
    }
}

const LIST: RustFunc = RustFunc(|args| RuntimeValue::List(args.to_vec()));

const CONS: RustFunc = RustFunc(|args| {
    if args.len() != 2 {
        panic!("cons must be called with two arguments");
    }
    match &args[1] {
        RuntimeValue::List(list) => {
            let mut new = list.clone();
            new.insert(0, args[0].clone());
            RuntimeValue::List(new)
        }
        _ => panic!("cons must be called with a list as the second argument"),
    }
});

const CAR: RustFunc = RustFunc(|args| {
    if args.len() != 1 {
        panic!("car must be called with one argument");
    }
    match &args[0] {
        RuntimeValue::List(list) => list[0].clone(),
        _ => panic!("car must be called with a list as the first argument"),
    }
});

const CDR: RustFunc = RustFunc(|args| {
    if args.len() != 1 {
        panic!("cdr must be called with one argument");
    }
    match &args[0] {
        RuntimeValue::List(list) => {
            let mut new = list.clone();
            new.remove(0);
            RuntimeValue::List(new)
        }
        _ => panic!("cdr must be called with a list as the first argument"),
    }
});

const FIRST: RustFunc = RustFunc(|args| {
    if args.len() != 1 {
        panic!("first must be called with one argument");
    };
    match &args[0] {
        RuntimeValue::List(list) => list[0].clone(),
        _ => panic!("first must be called with a list as the first argument"),
    }
});

const LAST: RustFunc = RustFunc(|args| {
    if args.len() != 1 {
        panic!("first must be called with one argument");
    };
    match &args[0] {
        RuntimeValue::List(list) => list.last().unwrap().clone(),
        _ => panic!("first must be called with a list as the first argument"),
    }
});

const EMPTYP: RustFunc = RustFunc(|args| {
    if args.len() != 1 {
        panic!("emptyp must be called with one argument");
    };
    match &args[0] {
        RuntimeValue::List(list) => RuntimeValue::Boolean(list.is_empty()),
        _ => panic!("emptyp must be called with a list as the first argument"),
    }
});

pub const BUILTINTS: [(&str, RustFunc); 7] = [
    ("cons", CONS),
    ("car", CAR),
    ("cdr", CDR),
    ("list", LIST),
    ("first", FIRST),
    ("last", LAST),
    ("emptyp", EMPTYP),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdr() {
        let args = vec![RuntimeValue::List(vec![
            RuntimeValue::Int(1),
            RuntimeValue::Int(2),
        ])];
        assert_eq!(
            CDR.eval(&args),
            RuntimeValue::List(vec![RuntimeValue::Int(2),])
        );
    }

    #[test]
    fn test_car() {
        let args = vec![RuntimeValue::List(vec![
            RuntimeValue::Int(1),
            RuntimeValue::Int(2),
        ])];
        assert_eq!(CAR.eval(&args), RuntimeValue::Int(1),);
    }

    #[test]
    fn test_cons() {
        let args = vec![
            RuntimeValue::Int(1),
            RuntimeValue::List(vec![RuntimeValue::Int(2), RuntimeValue::Int(3)]),
        ];
        assert_eq!(
            CONS.eval(&args),
            RuntimeValue::List(vec![
                RuntimeValue::Int(1),
                RuntimeValue::Int(2),
                RuntimeValue::Int(3),
            ])
        );
    }

    #[test]
    fn test_list() {
        let args = vec![
            RuntimeValue::Int(1),
            RuntimeValue::Int(2),
            RuntimeValue::Int(3),
        ];
        assert_eq!(
            LIST.eval(&args),
            RuntimeValue::List(vec![
                RuntimeValue::Int(1),
                RuntimeValue::Int(2),
                RuntimeValue::Int(3),
            ])
        );
    }
}
