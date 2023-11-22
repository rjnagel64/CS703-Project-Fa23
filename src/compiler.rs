

use crate::syntax::{Expr, BinOp, Var, Stmt, Block, Program};
use std::collections::HashMap;


#[derive(Debug, Clone, Copy)]
pub enum Insn {
    Halt,
    Literal(i64),
    Add,
    Sub,
    Mul,
    Lt,
    Gt,
    Print,
    Enter(usize),
    Exit(usize),
    GetLocal(usize),
    SetLocal(usize),
    Input(usize),
    Input2,
    Branch(isize),
    BranchZero(isize),
}

pub struct Compiler {
    code: Vec<Insn>,
    slots: HashMap<Var, usize>,
    num_slots: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { code: Vec::new(), slots: HashMap::new(), num_slots: 0 }
    }

    fn emit(&mut self, i: Insn) {
        self.code.push(i)
    }

    fn here(&self) -> usize {
        self.code.len()
    }

    fn branch_offset(&self, from: usize, to: usize) -> isize {
        (to as isize) - (from as isize)
    }

    pub fn output(self) -> Vec<Insn> {
        self.code
    }

    fn compile_exp(&mut self, e: &Expr) {
        match e {
            Expr::Var(x) => {
                let slot = self.slots.get(x).unwrap();
                self.emit(Insn::GetLocal(*slot));
            }
            Expr::Num(i) => self.emit(Insn::Literal(*i)),
            Expr::BinOp(b, e1, e2) => {
                self.compile_exp(e1);
                self.compile_exp(e2);
                match b {
                    BinOp::Add => self.emit(Insn::Add),
                    BinOp::Sub => self.emit(Insn::Sub),
                    BinOp::Mul => self.emit(Insn::Mul),
                    BinOp::Lt => self.emit(Insn::Lt),
                    BinOp::Gt => self.emit(Insn::Gt),
                }
            },
            Expr::Input(e) => {
                self.compile_exp(e);
                self.emit(Insn::Input2);
            }
        }
    }

    fn compile_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Assign(x, e) => {
                self.compile_exp(e);
                let slot = self.slots.get(x).unwrap();
                self.emit(Insn::SetLocal(*slot));
            },
            Stmt::Print(e) => {
                self.compile_exp(e);
                self.emit(Insn::Print)
            },
            Stmt::If(e, bt, bf) => {
                self.compile_exp(e);
                let branch = self.here();
                self.emit(Insn::BranchZero(0));

                self.compile_block(bt);
                let bt_end = self.here();
                self.emit(Insn::Branch(0));

                let bf_start = self.here();
                self.compile_block(bf);
                let bf_end = self.here();

                // Patch branches now that we know distance between the labels.
                self.code[branch] = Insn::BranchZero(self.branch_offset(branch, bf_start));
                self.code[bt_end] = Insn::Branch(self.branch_offset(bt_end, bf_end));
            }
            Stmt::While(e, b) => {
                let loop_start = self.here();
                self.compile_exp(e);
                let branch = self.here();
                self.emit(Insn::BranchZero(0));

                self.compile_block(b);
                let repeat = self.here();
                self.emit(Insn::Branch(0));
                let loop_end = self.here();

                self.code[branch] = Insn::BranchZero(self.branch_offset(branch, loop_end));
                self.code[repeat] = Insn::Branch(self.branch_offset(repeat, loop_start));
            }
        }
    }

    fn compile_block(&mut self, b : &Block) {
        for s in &b.0 {
            self.compile_stmt(s);
        }
    }

    pub fn compile_program(&mut self, p: &Program) {
        self.assign_slots(p);

        self.emit(Insn::Enter(self.num_slots));
        self.compile_block(&p.0);
        self.emit(Insn::Exit(self.num_slots));
        self.emit(Insn::Halt);
    }

    // Traverse the program to assign local slots to each distinct variable.
    // The implementation currently assigns each distinct variable its own slot, which is correct,
    // but not optimal. A smarter implementation would use live variable ranges to re-use slots
    // once their contents are no longer live, similar to register allocation (but without the need
    // to worry about register spilling.)
    fn assign_slots(&mut self, p: &Program) {
        self.assign_slots_block(&p.0);
    }

    fn assign_slots_block(&mut self, b: &Block) {
        for s in &b.0 {
            self.assign_slots_stmt(s);
        }
    }

    fn assign_slots_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Assign(x, _e) => {
                let entry = self.slots.entry(x.clone());
                // If this variable already has a slot, nothing needs to be done.
                // Otherwise, we need to assign a new slot.
                entry.or_insert_with(|| {
                    let i = self.num_slots;
                    self.num_slots += 1;
                    i
                });
            },
            Stmt::Print(_e) => {},
            Stmt::If(_e, bt, bf) => {
                self.assign_slots_block(bt);
                self.assign_slots_block(bf);
            }
            Stmt::While(_e, b) => {
                self.assign_slots_block(b);
            }
        }
    }
}



pub struct VM {
    stack: Vec<i64>,
    locals: Vec<i64>,
    code: Vec<Insn>,
    pc: usize, // index of current instruction in `code`
    fp: usize, // offset of current frame in `locals`
    args: Vec<i64>, // command-line arguments provided as inputs to the program
}

impl VM {
    pub fn new(code: Vec<Insn>, args: Vec<i64>) -> Self {
        VM { stack: Vec::new(), locals: Vec::new(), code: code, pc: 0, fp: 0, args: args }
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
            Insn::Sub => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(x - y);
            },
            Insn::Mul => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(x * y);
            },
            Insn::Lt => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(if x < y { 1 } else { 0 });
            },
            Insn::Gt => {
                let y = self.stack.pop().unwrap();
                let x = self.stack.pop().unwrap();
                self.stack.push(if x > y { 1 } else { 0 });
            },
            Insn::Print => {
                let x = self.stack.pop().unwrap();
                println!("{}", x);
            },
            Insn::Enter(n) => {
                self.locals.push(self.fp as i64); // hmm. Annoying cast.
                self.fp = self.locals.len();
                for _ in 0..n {
                    self.locals.push(0);
                }
            },
            Insn::Exit(n) => {
                for _ in 0..n {
                    self.locals.pop().unwrap();
                }
                self.fp = self.locals.pop().unwrap() as usize; // hmm. Annoying cast.
            },
            Insn::GetLocal(x) => {
                self.stack.push(self.locals[self.fp + x]);
            },
            Insn::SetLocal(x) => {
                self.locals[self.fp + x] = self.stack.pop().unwrap();
            },
            Insn::Input(x) => {
                let index = self.stack.pop().unwrap();
                self.locals[self.fp + x] = self.args[index as usize];
            },
            Insn::Input2 => {
                let index = self.stack.pop().unwrap();
                self.stack.push(self.args[index as usize]);
            },
            Insn::Branch(n) => return Some(self.pc.wrapping_add_signed(n)),
            Insn::BranchZero(n) => {
                let x = self.stack.pop().unwrap();
                if x == 0 {
                    return Some(self.pc.wrapping_add_signed(n));
                }
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

