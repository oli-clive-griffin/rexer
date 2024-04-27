use crate::sexpr::Sexpr;

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltIn {
    pub symbol: &'static str,
    eval: fn(&[Sexpr]) -> Result<Sexpr, String>,
}

impl BuiltIn {
    pub fn eval(&self, args: &[Sexpr]) -> Result<Sexpr, String> {
        (self.eval)(args)
    }
}

const LIST: BuiltIn = BuiltIn {
    symbol: "list",
    eval: |args| Ok(Sexpr::List {
        quasiquote: false,
        sexprs: args.to_vec(),
    }),
};

const CONS: BuiltIn = BuiltIn {
    symbol: "cons",
    eval: |args| {
        if args.len() != 2 {
            return Err("cons must be called with two arguments".to_string());
        }
        match &args[1] {
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => {
                let mut new = list.clone();
                new.insert(0, args[0].clone());
                Ok(Sexpr::List {
                    quasiquote: false,
                    sexprs: new,
                })
            }
            a => Err(format!("cons must be called with a list as the second argument, got {:#?}", a)),
        }
    },
};

const CAR: BuiltIn = BuiltIn {
    symbol: "car",
    eval: |args| {
        if args.len() != 1 {
            return Err("car must be called with one argument".to_string());
        }
        match &args[0] {
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => Ok(list[0].clone()),
            _ => Err("car must be called with a list as the first argument".to_string()),
        }
    },
};

const CDR: BuiltIn = BuiltIn {
    symbol: "cdr",
    eval: |args| {
        if args.len() != 1 {
            return Err(format!("cdr must be called with one argument, got {}", args.len()));
        }
        match &args[0] {
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => {
                let mut new = list.clone();
                new.remove(0);
                Ok(Sexpr::List {
                    quasiquote: false,
                    sexprs: new,
                })
            }
            _ => Err("cdr must be called with a list as the first argument".to_string()),
        }
    },
};

const ADD: BuiltIn = BuiltIn {
    symbol: "+",
    eval: |args| {
        let mut out = 0;
        for arg in args {
            match arg {
                Sexpr::Int(i) => out += i,
                // Sexpr::Float(i) => out += i as f64,
                _ => return Err("add must be called with a list of integers".to_string()),
            }
        }
        Ok(Sexpr::Int(out))
    },
};

const SUB: BuiltIn = BuiltIn {
    symbol: "-",
    eval: |args| {
        let mut init = match args[0] {
            Sexpr::Int(i) => i,
            // Sexpr::Float(i) => i as f64,
            _ => return Err("sub must be called with a list of integers".to_string()),
        };
        for arg in &args[1..] {
            match arg {
                Sexpr::Int(j) => init -= j,
                // Sexpr::Float(j) => init -= j as f64,
                _ => return Err("sub must be called with a list of integers".to_string()),
            }
        }
        Ok(Sexpr::Int(init))
    },
};

const MUL: BuiltIn = BuiltIn {
    symbol: "*",
    eval: |args| {
        let mut out = 1;
        for arg in args {
            match arg {
                Sexpr::Int(i) => out *= i,
                // Sexpr::Float(i) => out *= i as f64,
                _ => return Err("mul must be called with a list of integers".to_string()),
            }
        }
        Ok(Sexpr::Int(out))
    },
};

const DIV: BuiltIn = BuiltIn {
    symbol: "/",
    eval: |args| {
        let mut init = match args[0] {
            Sexpr::Int(i) => i,
            // Sexpr::Float(i) => i as f64,
            _ => return Err("div must be called with a list of integers".to_string()),
        };
        for arg in &args[1..] {
            match arg {
                Sexpr::Int(j) => init /= j,
                // Sexpr::Float(j) => init /= j as f64,
                _ => return Err("div must be called with a list of integers".to_string()),
            }
        }
        Ok(Sexpr::Int(init))
    },
};

const EMPTY: BuiltIn = BuiltIn {
    symbol: "empty?",
    eval: |args| {
        if args.len() != 1 {
            return Err("empty must be called with one argument".to_string());
        }
        match &args[0] {
            Sexpr::List {
                quasiquote: false,
                sexprs: list,
            } => Ok(Sexpr::Bool(list.is_empty())),
            _ => Err("empty must be called with a list as the first argument".to_string()),
        }
    },
};

const INC: BuiltIn = BuiltIn {
    symbol: "inc",
    eval: |args| {
        if args.len() != 1 {
            return Err("inc must be called with one argument".to_string());
        }
        match &args[0] {
            Sexpr::Int(i) => Ok(Sexpr::Int(i + 1)),
            // Sexpr::Float(i) => Sexpr::Float(i + 1.0),
            _ => Err("inc must be called with an integer".to_string()),
        }
    },
};


const PRINT: BuiltIn = BuiltIn {
    symbol: "print",
    eval: |args| {
        for arg in args {
            println!("{}", arg);
        }
        Ok(Sexpr::Bool(true)) // TODO introduce a new type for void / unit
    },
};

const EQ: BuiltIn = BuiltIn {
    symbol: "=",
    eval: |args| {
        if args.len() != 2 {
            return Err("= must be called with two arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (Sexpr::Int(i), Sexpr::Int(j)) => Ok(Sexpr::Bool(i == j)),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => Err("= must be called with two integers".to_string()),
        }
    }
};

const GT: BuiltIn = BuiltIn {
    symbol: ">",
    eval: |args| {
        if args.len() != 2 {
            return Err("= must be called with two arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (Sexpr::Int(i), Sexpr::Int(j)) => Ok(Sexpr::Bool(i > j)),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => Err("= must be called with two integers".to_string()),
        }
    }
};

const LT: BuiltIn = BuiltIn {
    symbol: "<",
    eval: |args| {
        if args.len() != 2 {
            return Err("= must be called with two arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (Sexpr::Int(i), Sexpr::Int(j)) => Ok(Sexpr::Bool(i < j)),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => Err("= must be called with two integers".to_string()),
        }
    }
};

pub const BUILT_INS: [BuiltIn; 14] = [CONS, CAR, CDR, LIST, ADD, SUB, MUL, DIV, EMPTY, INC, PRINT, EQ, GT, LT];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cons() -> Result<(), String> {
        let args = vec![
            Sexpr::Int(1),
            Sexpr::List {
                quasiquote: false,
                sexprs: vec![Sexpr::Int(2), Sexpr::Int(3)],
            },
        ];
        assert_eq!(
            CONS.eval(&args)?,
            Sexpr::List {
                quasiquote: false,
                sexprs: vec![Sexpr::Int(1), Sexpr::Int(2), Sexpr::Int(3),]
            }
        );
        Ok(())
    }
}
