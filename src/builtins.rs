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
    eval: |args| Sexpr::List {
        quasiquote: false,
        sexprs: args.to_vec(),
    },
};

const CONS: BuiltIn = BuiltIn {
    symbol: "cons",
    eval: |args| {
        if args.len() != 2 {
            panic!("cons must be called with two arguments");
        }
        match &args[1] {
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => {
                let mut new = list.clone();
                new.insert(0, args[0].clone());
                Sexpr::List {
                    quasiquote: false,
                    sexprs: new,
                }
            }
            a => panic!("cons must be called with a list as the second argument, got {:#?}", a),
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
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => list[0].clone(),
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
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => {
                let mut new = list.clone();
                new.remove(0);
                Sexpr::List {
                    quasiquote: false,
                    sexprs: new,
                }
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
        let mut init = match args[0] {
            Sexpr::Int(i) => i,
            // Sexpr::Float(i) => i as f64,
            _ => panic!("sub must be called with a list of integers"),
        };
        for i in 1..args.len() {
            match args[i] {
                Sexpr::Int(j) => init -= j,
                // Sexpr::Float(j) => init -= j as f64,
                _ => panic!("sub must be called with a list of integers"),
            }
        }
        Sexpr::Int(init)
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
        let mut init = match args[0] {
            Sexpr::Int(i) => i,
            // Sexpr::Float(i) => i as f64,
            _ => panic!("div must be called with a list of integers"),
        };
        for i in 1..args.len() {
            match args[i] {
                Sexpr::Int(j) => init /= j,
                // Sexpr::Float(j) => init /= j as f64,
                _ => panic!("div must be called with a list of integers"),
            }
        }
        Sexpr::Int(init)
    },
};

const EMPTY: BuiltIn = BuiltIn {
    symbol: "empty?",
    eval: |args| {
        if args.len() != 1 {
            panic!("empty must be called with one argument");
        }
        match &args[0] {
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => Sexpr::Bool(list.is_empty()),
            _ => panic!("empty must be called with a list as the first argument"),
        }
    },
};

const INC: BuiltIn = BuiltIn {
    symbol: "inc",
    eval: |args| {
        if args.len() != 1 {
            panic!("inc must be called with one argument");
        }
        match &args[0] {
            Sexpr::Int(i) => Sexpr::Int(i + 1),
            // Sexpr::Float(i) => Sexpr::Float(i + 1.0),
            _ => panic!("inc must be called with an integer"),
        }
    },
};


const PRINT: BuiltIn = BuiltIn {
    symbol: "print",
    eval: |args| {
        for arg in args {
            println!("{}", arg);
        }
        Sexpr::Bool(true) // TODO introduce a new type for void / unit
    },
};

const EQ: BuiltIn = BuiltIn {
    symbol: "=",
    eval: |args| {
        if args.len() != 2 {
            panic!("= must be called with two arguments");
        }
        match (&args[0], &args[1]) {
            (Sexpr::Int(i), Sexpr::Int(j)) => Sexpr::Bool(i == j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => panic!("= must be called with two integers"),
        }
    }
};

const GT: BuiltIn = BuiltIn {
    symbol: ">",
    eval: |args| {
        if args.len() != 2 {
            panic!("= must be called with two arguments");
        }
        match (&args[0], &args[1]) {
            (Sexpr::Int(i), Sexpr::Int(j)) => Sexpr::Bool(i > j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => panic!("= must be called with two integers"),
        }
    }
};

const LT: BuiltIn = BuiltIn {
    symbol: "<",
    eval: |args| {
        if args.len() != 2 {
            panic!("= must be called with two arguments");
        }
        match (&args[0], &args[1]) {
            (Sexpr::Int(i), Sexpr::Int(j)) => Sexpr::Bool(i < j),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => panic!("= must be called with two integers"),
        }
    }
};

pub const BUILTINTS: [BuiltIn; 14] = [CONS, CAR, CDR, LIST, ADD, SUB, MUL, DIV, EMPTY, INC, PRINT, EQ, GT, LT];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cons() {
        let args = vec![
            Sexpr::Int(1),
            Sexpr::List {
                quasiquote: false,
                sexprs: vec![Sexpr::Int(2), Sexpr::Int(3)],
            },
        ];
        assert_eq!(
            CONS.eval(&args),
            Sexpr::List {
                quasiquote: false,
                sexprs: vec![Sexpr::Int(1), Sexpr::Int(2), Sexpr::Int(3),]
            }
        );
    }
}
