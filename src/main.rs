
use std::collections::HashMap;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser);

mod syntax;

use syntax::{Expr, BinOp, Stmt, Block, Program};

#[derive(Debug, Clone, Copy)]
enum Insn {
    Halt,
    Literal(i64),
    Add,
    Mul,
    Print,
    Enter(usize), // create a stack frame with n locals
    Exit, // pop the current stack frame
    GetLocal(usize), // retrieve local variable from current frame
    SetLocal(usize), // update local variable in current frame
}

type Location = ();

struct Compiler {
    code: Vec<Insn>,
    scope: HashMap<String, Location>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { code: Vec::new(), scope: HashMap::new() }
    }

    fn emit(&mut self, i: Insn) {
        self.code.push(i)
    }

    fn output(self) -> Vec<Insn> {
        self.code
    }

    fn compile_exp(&mut self, e: &Expr) {
        match e {
            Expr::Var(x) => todo!("compile var exp"),
            Expr::Num(i) => self.emit(Insn::Literal(*i)),
            Expr::BinOp(b, e1, e2) => {
                self.compile_exp(e1);
                self.compile_exp(e2);
                match b {
                    BinOp::Add => self.emit(Insn::Add),
                    BinOp::Mul => self.emit(Insn::Mul),
                }
            },
        }
    }

    fn compile_stmt(&mut self, s: &Stmt) {
        match s {
            // TODO: Figure out how to assign slots in the local frame.  I think I will need to
            // thread through some extra state, that maps variable names to stack slots (This is
            // also necessary to compile variable expressions) Generating a new slot for every
            // assigned variable is technically *correct*... but certainly not optimally efficient.
            // Trying to minimize the number of slots used basically boils down to register
            // allocation, but without needing to worry about register spills.  (minimum number of
            // local slots required == max number of variables simulatenously live?  Sounds
            // reasonable. Compute liveness with reaching definition dataflow? Maybe.)
            Stmt::Assign(x, e) => todo!("compile assign stmt"),
            Stmt::Print(e) => {
                self.compile_exp(e);
                self.emit(Insn::Print)
            },
            Stmt::If(e, bt, bf) => todo!("compile if stmt"),
            Stmt::While(e, b) => todo!("compile if stmt"),
        }
    }

    fn compile_block(&mut self, b : &Block) {
        for s in &b.0 {
            self.compile_stmt(s);
        }
    }

    fn compile_program(&mut self, p: &Program) {
        // TODO: Assign slots to all variables
        self.compile_block(&p.0);
        self.emit(Insn::Halt);
    }

}


struct VM {
    stack: Vec<i64>,
    pc: usize,
    code: Vec<Insn>,
    // TODO: this 'locals' stack isn't quite right.
    // Actual CPUs keep a single 'frame pointer'/'base pointer' in a register, and local references
    // are made as
    // offsets to the FP/BP
    // 'Enter n' pushes current FP, sets FP = FP+1 then expands stack by n slots
    // 'Exit' sets FP to previous FP, discards slots from current frame
    locals: Vec<i64>,
}

impl VM {
    pub fn new(code: Vec<Insn>) -> Self {
        VM { stack: Vec::new(), pc: 0, code: code, locals: Vec::new() }
    }

    fn step(&mut self) -> Option<usize> {
        let insn = self.code[self.pc];
        match insn {
            Insn::Halt => return None,
            Insn::Literal(i) => {
                self.stack.push(i);
            },
            Insn::Add => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(x + y);
            },
            Insn::Mul => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(x * y);
            },
            Insn::Print => {
                let x = self.stack.pop().unwrap();
                println!("{}", x);
            },
            Insn::Enter(n) => {
                for _i in 0..n {
                    self.locals.push(0 as i64);
                }
                self.locals.push(n as i64);
            },
            Insn::Exit => {
                let n = self.locals.pop().unwrap();
                for _i in 0..n {
                    self.locals.pop();
                }
            },
            Insn::GetLocal(x) => {
                // len() - 1 contains number of locals in this block.
                // len() - 1 - 1 contains local #1
                // len() - 1 - i contains local #i
                let index = self.locals.len() - 1 - x;
                let val = self.locals[index];
                self.stack.push(val);
            },
            Insn::SetLocal(x) => {
                let index = self.locals.len() - 1 - x;
                let val = self.stack.pop().unwrap();
                self.locals[index] = val;
            },
        }
        Some(self.pc + 1)
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
    let src = "(3 + 4) * 5";
    let e = p.parse(src).expect("valid syntax");
    let b = Block(vec![Stmt::Print(Box::new(e))]);
    let p = Program(b);

    let mut com = Compiler::new();
    com.compile_program(&p);

    let code = com.output();
    println!("{:?}", code);

    let mut vm = VM::new(code);

    vm.execute();
    vm.dump_state();
}
