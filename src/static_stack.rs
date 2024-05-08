use std::{array::from_fn, fmt::Display};

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
        true
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
        s.push(']');
        write!(f, "{}", s)
    }
}

impl<T: Default /* + Copy */ + Clone, const MAX: usize> Default for StaticStack<T, MAX> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default /* + Copy */ + Clone, const MAX: usize> StaticStack<T, MAX> {
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.stack.as_mut_ptr()
    }

    pub fn new() -> Self {
        Self {
            stack: from_fn(|_i| Default::default()), // [Default::default(); MAX],
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
        let value = self.stack[self.ptr as usize].clone();
        self.stack[self.ptr as usize] = Default::default();
        self.ptr -= 1;
        Some(value)
    }

    pub fn peek_back(&self, back: usize) -> Option<T> {
        let idx = self.ptr - back as i32;
        if idx < 0 {
            return None;
        }
        Some(self.stack[idx as usize].clone())
    }

    pub fn at(&self, idx: usize) -> Option<&T> {
        self.stack.get(idx)
    }

    pub fn at_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.stack.get_mut(idx)
    }

    pub fn pop_n(&mut self, n: usize) -> Option<Vec<T>> {
        if n > self.len() {
            return None;
        }
        let start = self.ptr as usize + 1 - n; // plus one first to prevent underflow
        let end = self.ptr as usize + 1;
        let vec = self.stack[start..end].into_iter().cloned().collect();
        self.ptr -= n as i32;
        Some(vec)
    }

    pub fn peek_top(&self) -> Option<&T> {
        self.at(self.ptr as usize)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        (self.ptr + 1) as usize
    }
}

impl<T: Default /* + Copy */ + Clone, const MAX: usize, const N: usize> From<[T; N]>
    for StaticStack<T, MAX>
{
    fn from(values: [T; N]) -> Self {
        let mut stack = Self::new();
        for value in values {
            stack.push(value);
        }
        stack
    }
}
