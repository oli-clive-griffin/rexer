use std::ops;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    // Function ...
}

impl ops::Add for RuntimeValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => RuntimeValue::Int(a + b),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a + b),
            (RuntimeValue::Int(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a as f64 + b),
            (RuntimeValue::Float(a), RuntimeValue::Int(b)) => RuntimeValue::Float(a + b as f64),
            (RuntimeValue::String(a), RuntimeValue::String(b)) => RuntimeValue::String(a + &b),
            (s, r) => panic!("Cannot add {:?} and {:?}", s, r),
        }
    }
}

impl ops::Mul for RuntimeValue {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => RuntimeValue::Int(a * b),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a * b),
            (RuntimeValue::Int(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a as f64 * b),
            (RuntimeValue::Float(a), RuntimeValue::Int(b)) => RuntimeValue::Float(a * b as f64),
            (s, r) => panic!("Cannot multiply {:?} and {:?}", s, r),
        }
    }
}

impl ops::Sub for RuntimeValue {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => RuntimeValue::Int(a - b),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a - b),
            (RuntimeValue::Int(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a as f64 - b),
            (RuntimeValue::Float(a), RuntimeValue::Int(b)) => RuntimeValue::Float(a - b as f64),
            (s, r) => panic!("Cannot subtract {:?} and {:?}", s, r),
        }
    }
}

impl ops::Div for RuntimeValue {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => RuntimeValue::Int(a / b),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a / b),
            (RuntimeValue::Int(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a as f64 / b),
            (RuntimeValue::Float(a), RuntimeValue::Int(b)) => RuntimeValue::Float(a / b as f64),
            (s, r) => panic!("Cannot divide {:?} and {:?}", s, r),
        }
    }
}
