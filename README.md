
# CS 703 Term Project - Fall 2023

This project involves implementing a simple imperative language and using
egraphs to optimize it.

TODO: Fill in details from the project proposal.

## Examples:

```
cargo run factorial.prog 10
   Compiling project v0.1.0 (/.../.../...)
    Finished dev [unoptimized + debuginfo] target(s) in 0.42s
     Running `target/debug/project factorial.prog 10`
--- Compiled bytecode: ---
[Enter(2), Input(0), Literal(1), SetLocal(1), GetLocal(0), Literal(0), Gt, BranchZero(10), GetLocal(1),
 GetLocal(0), Mul, SetLocal(1), GetLocal(0), Literal(1), Sub, SetLocal(0), Branch(-12), GetLocal(1), Print, Exit(2), Halt]
--- Results: ---
3628800
pc = 20
stack = []
```

```
cargo run fibonacci.prog
--- Compiled bytecode: ---
((omitted))
--- Results: ---
55
pc = 27
stack = []
```
