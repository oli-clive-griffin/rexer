use std::fmt::Display;

#[derive(Debug)]
pub struct StaticStack<T, const MAX: usize> {
    stack: [T; MAX],
    pub ptr: i32, // needs to be i to allow -1
}

impl<T: PartialEq, const MAX: usize> PartialEq for StaticStack<T, MAX> {
    fn eq(&self, other: &Self) -> bool {
        if self.ptr != other.ptr {
            return false;
        }
        for i in 0..=self.ptr as usize {
            if self.stack[i] != other.stack[i] {
                return false;
            }
        }
        return true;
    }
}

impl<T: Display, const MAX: usize> Display for StaticStack<T, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::from("[");
        let max = if self.ptr < 0 {
            return write!(f, "[]");
        } else {
            self.ptr as usize
        };
        for i in 0..max {
            s.push_str(&format!("{}, ", self.stack[i]));
        }
        s.push_str(&format!("{}", self.stack[self.ptr as usize]));
        s.push_str("]");
        write!(f, "{}", s)
    }
}

impl<T: Default + Copy, const MAX: usize> StaticStack<T, MAX> {
    pub fn new() -> Self {
        Self {
            stack: [Default::default(); MAX],
            ptr: -1,
        }
    }

    pub fn push(&mut self, value: T) {
        self.ptr += 1;
        self.stack[self.ptr as usize] = value;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.ptr == -1 {
            return None;
        }
        let value = self.stack[self.ptr as usize];
        self.ptr -= 1;
        return Some(value);
    }

    pub fn peek_back(&self, back: usize) -> Option<T> {
        let idx = self.ptr - back as i32;
        if idx < 0 {
            return None;
        }
        return Some(self.stack[idx as usize]);
    }


    pub fn at(&self, idx: usize) -> Option<&T> {
        if idx > self.ptr as usize {
            return None;
        }
        return Some(&self.stack[idx]);
    }
}
