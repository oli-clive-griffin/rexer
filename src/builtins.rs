use crate::runtime_value::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltIn {
    pub name: &'static str,
    eval: fn(&[RuntimeValue]) -> RuntimeValue,
}

impl BuiltIn {
    pub fn eval(&self, args: &[RuntimeValue]) -> RuntimeValue {
        (self.eval)(args)
    }
}

const LIST: BuiltIn = BuiltIn {
    name: "list",
    eval: |args| RuntimeValue::List(args.to_vec()),
};

const CONS: BuiltIn = BuiltIn {
    name: "cons",
    eval: |args| {
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
    },
};

const CAR: BuiltIn = BuiltIn {
    name: "car",
    eval: |args| {
        if args.len() != 1 {
            panic!("car must be called with one argument");
        }
        match &args[0] {
            RuntimeValue::List(list) => list[0].clone(),
            _ => panic!("car must be called with a list as the first argument"),
        }
    },
};

const CDR: BuiltIn = BuiltIn {
    name: "cdr",
    eval: |args| {
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
    },
};

pub const BUILTINTS: [BuiltIn; 4] = [CONS, CAR, CDR, LIST];

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_quote() {
    //     let args = vec![RuntimeValue::List(vec![
    //         RuntimeValue::Int(1),
    //         RuntimeValue::Int(2),
    //         RuntimeValue::Int(3),
    //     ])];
    //     assert_eq!(
    //         QUOTE.eval(&args),
    //         RuntimeValue::List(vec![
    //             RuntimeValue::Int(1),
    //             RuntimeValue::Int(2),
    //             RuntimeValue::Int(3),
    //         ])
    //     );
    // }

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
}
