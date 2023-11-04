
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser);

mod syntax;

use syntax::{Expr, BinOp, Stmt, Block};

#[derive(Debug, Clone, Copy)]
enum Insn {
    Literal(i64),
    Add,
    Mul,
    Print,
}

fn compile_exp(e: &Expr, code: &mut Vec<Insn>) {
    // write output bytes to `code`.
    match e {
        Expr::Var(x) => todo!("compile var exp"),
        Expr::Num(i) => code.push(Insn::Literal(*i)),
        Expr::BinOp(b, e1, e2) => {
            compile_exp(e1, code);
            compile_exp(e2, code);
            match b {
                BinOp::Add => code.push(Insn::Add),
                BinOp::Mul => code.push(Insn::Mul),
            }
        },
    }
}

fn compile_block(b: &Block, code: &mut Vec<Insn>) {
    for s in &b.0 {
        compile_stmt(s, code);
    }
}

fn compile_stmt(s: &Stmt, code: &mut Vec<Insn>) {
    match s {
        // How to store local variables at runtime?
        // With a stack of "Frame", where each Frame has a vector of slots for locals?
        Stmt::Assign(x, e) => todo!("compile assign stmt"),
        Stmt::Print(e) => {
            compile_exp(e, code);
            code.push(Insn::Print)
        },
        Stmt::If(e, bt, bf) => todo!("compile if stmt"),
        Stmt::While(e, b) => todo!("compile if stmt"),
    }
}

// Hmm. Need a `compile_program` function, that inserts a final "Halt" instruction so that the pc
// doesn't run off the end of the code segment.

struct VM {
    stack: Vec<i64>,
    pc: usize,
    code: Vec<Insn>,
}

impl VM {
    pub fn new(code: Vec<Insn>) -> Self {
        VM { stack: Vec::new(), pc: 0, code: code }
    }

    fn step(&mut self) -> Option<usize> {
        let insn = self.code[self.pc];
        match insn {
            Insn::Literal(i) => { self.stack.push(i); Some(self.pc + 1) },
            Insn::Add => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(x + y);
                Some(self.pc + 1)
            },
            Insn::Mul => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(x * y);
                Some(self.pc + 1)
            },
            Insn::Print => {
                let x = self.stack.pop().unwrap();
                println!("{}", x);
                Some(self.pc + 1)
            },
        }
    }
    
    pub fn execute(&mut self) {
        self.pc = 0;
        while let Some(new_pc) = self.step() {
            self.pc = new_pc;
        }
    }

    pub fn dump_state(&self) {
        println!("pc = {}", self.pc);
        println!("stack = {:?}", self.stack);
    }
}

fn main() {
    println!("Hello, world!");

    let p = parser::ExprParser::new();
    // let src = "(3 + x) * 5";
    let src = "(3 + 4) * 5";
    let e = p.parse(src).expect("valid syntax");

    let mut code = Vec::new();
    compile_exp(&e, &mut code);
    println!("{:?}", code);

    let mut vm = VM::new(code);

    vm.execute();
    vm.dump_state();
}
