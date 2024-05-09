use crate::sexpr::LispValue;

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltIn {
    pub symbol: &'static str,
    eval: fn(&[LispValue]) -> Result<LispValue, String>,
}

impl BuiltIn {
    pub fn eval(&self, args: &[LispValue]) -> Result<LispValue, String> {
        (self.eval)(args)
    }
}

const LIST: BuiltIn = BuiltIn {
    symbol: "list",
    eval: |args| Ok(LispValue::List(args.to_vec())),
};

const CONS: BuiltIn = BuiltIn {
    symbol: "cons",
    eval: |args| {
        if args.len() != 2 {
            return Err("cons must be called with two arguments".to_string());
        }
        match &args[1] {
            LispValue::List(list) => {
                let mut new = list.clone();
                new.insert(0, args[0].clone());
                Ok(LispValue::List(new))
            }
            a => Err(format! {
                "cons must be called with a list as the second argument, got {:#?}",
                a
            }),
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
            LispValue::List(list) => Ok(list[0].clone()),
            _ => Err("car must be called with a list as the first argument".to_string()),
        }
    },
};

const CDR: BuiltIn = BuiltIn {
    symbol: "cdr",
    eval: |args| {
        if args.len() != 1 {
            return Err(format!(
                "cdr must be called with one argument, got {}",
                args.len()
            ));
        }
        match &args[0] {
            LispValue::List(list) => {
                let mut new = list.clone();
                new.remove(0);
                Ok(LispValue::List(new))
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
                LispValue::Int(i) => out += i,
                // Sexpr::Float(i) => out += i as f64,
                _ => return Err("add must be called with a list of integers".to_string()),
            }
        }
        Ok(LispValue::Int(out))
    },
};

const SUB: BuiltIn = BuiltIn {
    symbol: "-",
    eval: |args| {
        let mut init = match args[0] {
            LispValue::Int(i) => i,
            // Sexpr::Float(i) => i as f64,
            _ => return Err("sub must be called with a list of integers".to_string()),
        };
        for arg in &args[1..] {
            match arg {
                LispValue::Int(j) => init -= j,
                // Sexpr::Float(j) => init -= j as f64,
                _ => return Err("sub must be called with a list of integers".to_string()),
            }
        }
        Ok(LispValue::Int(init))
    },
};

const MUL: BuiltIn = BuiltIn {
    symbol: "*",
    eval: |args| {
        let mut out = 1;
        for arg in args {
            match arg {
                LispValue::Int(i) => out *= i,
                // Sexpr::Float(i) => out *= i as f64,
                _ => return Err("mul must be called with a list of integers".to_string()),
            }
        }
        Ok(LispValue::Int(out))
    },
};

const DIV: BuiltIn = BuiltIn {
    symbol: "/",
    eval: |args| {
        let mut init = match args[0] {
            LispValue::Int(i) => i,
            // Sexpr::Float(i) => i as f64,
            _ => return Err("div must be called with a list of integers".to_string()),
        };
        for arg in &args[1..] {
            match arg {
                LispValue::Int(j) => init /= j,
                // Sexpr::Float(j) => init /= j as f64,
                _ => return Err("div must be called with a list of integers".to_string()),
            }
        }
        Ok(LispValue::Int(init))
    },
};

const EMPTY: BuiltIn = BuiltIn {
    symbol: "empty?",
    eval: |args| {
        if args.len() != 1 {
            return Err("empty must be called with one argument".to_string());
        }
        match &args[0] {
            LispValue::List(list) => Ok(LispValue::Bool(list.is_empty())),
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
            LispValue::Int(i) => Ok(LispValue::Int(i + 1)),
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
        Ok(LispValue::Nil)
    },
};

const EQ: BuiltIn = BuiltIn {
    symbol: "=",
    eval: |args| {
        if args.len() != 2 {
            return Err("= must be called with two arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (LispValue::Int(i), LispValue::Int(j)) => Ok(LispValue::Bool(i == j)),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => Err("= must be called with two integers".to_string()),
        }
    },
};

const GT: BuiltIn = BuiltIn {
    symbol: ">",
    eval: |args| {
        if args.len() != 2 {
            return Err("= must be called with two arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (LispValue::Int(i), LispValue::Int(j)) => Ok(LispValue::Bool(i > j)),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => Err("= must be called with two integers".to_string()),
        }
    },
};

const LT: BuiltIn = BuiltIn {
    symbol: "<",
    eval: |args| {
        if args.len() != 2 {
            return Err("= must be called with two arguments".to_string());
        }
        match (&args[0], &args[1]) {
            (LispValue::Int(i), LispValue::Int(j)) => Ok(LispValue::Bool(i < j)),
            // (Sexpr::Float(i), Sexpr::Float(j)) => Sexpr::Bool(i == j),
            _ => Err("= must be called with two integers".to_string()),
        }
    },
};

pub const BUILT_INS: [BuiltIn; 14] = [
    CONS, CAR, CDR, LIST, ADD, SUB, MUL, DIV, EMPTY, INC, PRINT, EQ, GT, LT,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cons() -> Result<(), String> {
        let args = vec![
            LispValue::Int(1),
            LispValue::List(vec![LispValue::Int(2), LispValue::Int(3)]),
        ];
        assert_eq!(
            CONS.eval(&args)?,
            LispValue::List(vec![LispValue::Int(1), LispValue::Int(2), LispValue::Int(3),])
        );
        Ok(())
    }
}
