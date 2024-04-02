use crate::parser::Sexpr;

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltIn {
    // pub name: &'static str,
    pub symbol: &'static str,
    eval: fn(&[Sexpr]) -> Sexpr,
}

impl BuiltIn {
    pub fn eval(&self, args: &[Sexpr]) -> Sexpr {
        (self.eval)(args)
    }
}

const LIST: BuiltIn = BuiltIn {
    symbol: "list",
    eval: |args| Sexpr::List(args.to_vec()),
};

const CONS: BuiltIn = BuiltIn {
    symbol: "cons",
    eval: |args| {
        if args.len() != 2 {
            panic!("cons must be called with two arguments");
        }
        match &args[1] {
            Sexpr::List(list) => {
                let mut new = list.clone();
                new.insert(0, args[0].clone());
                Sexpr::List(new)
            }
            _ => panic!("cons must be called with a list as the second argument"),
        }
    },
};

const CAR: BuiltIn = BuiltIn {
    symbol: "car",
    eval: |args| {
        if args.len() != 1 {
            panic!("car must be called with one argument");
        }
        match &args[0] {
            Sexpr::List(list) => list[0].clone(),
            _ => panic!("car must be called with a list as the first argument"),
        }
    },
};

const CDR: BuiltIn = BuiltIn {
    symbol: "cdr",
    eval: |args| {
        if args.len() != 1 {
            panic!("cdr must be called with one argument, got {}", args.len());
        }
        match &args[0] {
            Sexpr::List(list) => {
                let mut new = list.clone();
                new.remove(0);
                Sexpr::List(new)
            }
            _ => panic!("cdr must be called with a list as the first argument"),
        }
    },
};

const ADD: BuiltIn = BuiltIn {
    symbol: "+",
    eval: |args| {
        let out = args.iter().fold(0, |acc, x| match x {
            Sexpr::Int(i) => acc + i,
            // Sexpr::Float(i) => acc as f64 + i,
            _ => panic!("add must be called with a list of integers"),
        });
        Sexpr::Int(out)
    },
};

const SUB: BuiltIn = BuiltIn {
    symbol: "-",
    eval: |args| {
        let out = args.iter().fold(0, |acc, x| match x {
            Sexpr::Int(i) => acc - i,
            // Sexpr::Float(i) => acc as f64 - i,
            _ => panic!("sub must be called with a list of integers"),
        });
        Sexpr::Int(out)
    },
};

const MUL: BuiltIn = BuiltIn {
    symbol: "*",
    eval: |args| {
        let out = args.iter().fold(1, |acc, x| match x {
            Sexpr::Int(i) => acc * i,
            // Sexpr::Float(i) => acc as f64 * i,
            _ => panic!("mul must be called with a list of integers"),
        });
        Sexpr::Int(out)
    },
};

const DIV: BuiltIn = BuiltIn {
    symbol: "/",
    eval: |args| {
        let out = args.iter().fold(1, |acc, x| match x {
            Sexpr::Int(i) => acc / i,
            // Sexpr::Float(i) => acc as f64 / i,
            _ => panic!("div must be called with a list of integers"),
        });
        Sexpr::Int(out)
    },
};

const EMPTY: BuiltIn = BuiltIn {
    symbol: "empty?",
    eval: |args| {
        if args.len() != 1 {
            panic!("empty must be called with one argument");
        }
        match &args[0] {
            Sexpr::List(list) => Sexpr::Bool(list.is_empty()),
            _ => panic!("empty must be called with a list as the first argument"),
        }
    },
};

pub const BUILTINTS: [BuiltIn; 9] = [CONS, CAR, CDR, LIST, EMPTY, ADD, SUB, MUL, DIV];

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_quote() {
    //     let args = vec![Sexpr::List(vec![
    //         Sexpr::Int(1),
    //         Sexpr::Int(2),
    //         Sexpr::Int(3),
    //     ])];
    //     assert_eq!(
    //         QUOTE.eval(&args),
    //         Sexpr::List(vec![
    //             Sexpr::Int(1),
    //             Sexpr::Int(2),
    //             Sexpr::Int(3),
    //         ])
    //     );
    // }

    #[test]
    fn test_cons() {
        let args = vec![
            Sexpr::Int(1),
            Sexpr::List(vec![Sexpr::Int(2), Sexpr::Int(3)]),
        ];
        assert_eq!(
            CONS.eval(&args),
            Sexpr::List(vec![
                Sexpr::Int(1),
                Sexpr::Int(2),
                Sexpr::Int(3),
            ])
        );
    }
}
