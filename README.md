# Rusp

This is technically 2 lisp interpreters in one repo, which share a lexer and parser: a tree-walker, and bytecode-compiler/vm. The following will be focussed on the bytecode compiler/vm:

## Features
- [x] variable declarations
- [x] function declarations
- [x] recursion
- [x] closures
- [x] first-class functions
- [x] printing
- [x] quoting (not super stable but basically works)
- [x] conditionals
- [x] basic arithmetic
- [x] basic list operations: cons, car, cdr etc.
- [x] lambdas (via `fn`)
- [ ] garbage collection
- [ ] macros (the tree-walker has them, but the bytecode compiler/vm doesn't yet)

## Usage
Assuming you have the rust toolchain installed:
```bash
# the `--bin ruspc` is necessary to run the bytecode compiler/vm, use `--bin ruspt` to run the tree-walker

# run the repl
cargo run --bin ruspc

# run a file
cargo run --bin ruspc -- <path-to-file>
```
