
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser);

mod syntax;

use syntax::{Expr, BinOp, Stmt, Block};

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

// Hmm. Instead of free functions, make compile_* methods on a 'struct Compiler'?
// Yes. The compiler will include 'code' as a field so I don't have to pass it around, and probably
// also things like a symbol table for compiling assignments.
fn compile_exp(e: &Expr, code: &mut Vec<Insn>) {
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
    // compute slots for local variables, emit 'Enter' and 'Exit' instructions?
    for s in &b.0 {
        compile_stmt(s, code);
    }
}

fn compile_stmt(s: &Stmt, code: &mut Vec<Insn>) {
    match s {
        // TODO: Figure out how to assign slots in the local frame.
        // I think I will need to thread through some extra state, that maps variable names to
        // stack slots (This is also necessary to compile variable expressions)
        // Generating a new slot for every assigned variable is technically *correct*... but
        // certainly not optimally efficient. Trying to minimize the number of slots used basically
        // boils down to register allocation, but without needing to worry about register spills.
        // (minimum number of local slots required == max number of variables simulatenously live?
        // Sounds reasonable. Compute liveness with reaching definition dataflow? Maybe.)
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
                for i in 0..n {
                    self.locals.push(0 as i64);
                }
                self.locals.push(n as i64);
            },
            Insn::Exit => {
                let n = self.locals.pop().unwrap();
                for i in 0..n {
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
    // let src = "(3 + x) * 5";
    let src = "(3 + 4) * 5";
    let e = p.parse(src).expect("valid syntax");

    let mut code = Vec::new();
    compile_exp(&e, &mut code);
    code.push(Insn::Print);
    code.push(Insn::Halt);
    println!("{:?}", code);

    let mut vm = VM::new(code);

    vm.execute();
    vm.dump_state();
}
