
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
    Sub,
    Mul,
    Lt,
    Gt,
    Print,
    Enter(usize), // create a stack frame with n locals
    Exit, // pop the current stack frame
    GetLocal(usize), // retrieve local variable from current frame
    SetLocal(usize), // update local variable in current frame
    Branch(isize),
    BranchZero(isize),
}

struct Compiler {
    code: Vec<Insn>,
    slots: HashMap<String, usize>,
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

    fn output(self) -> Vec<Insn> {
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
        }
    }

    fn compile_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Assign(x, e) => {
                self.compile_exp(e);
                let slot = self.slots.get(x).unwrap();
                self.emit(Insn::SetLocal(*slot));
            }
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

                // aargh. can't patch loop_end because code[loop_end] is out of bounds (there's no
                // actual instruction at loop_end)

                self.code[branch] = Insn::BranchZero(self.branch_offset(branch, loop_end));
                // will be negative.
                self.code[repeat] = Insn::Branch(self.branch_offset(repeat, loop_start));
            }
        }
    }

    fn compile_block(&mut self, b : &Block) {
        for s in &b.0 {
            self.compile_stmt(s);
        }
    }

    fn compile_program(&mut self, p: &Program) {
        self.assign_slots(p);

        self.emit(Insn::Enter(self.num_slots));
        self.compile_block(&p.0);
        self.emit(Insn::Exit);
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

fn main() {
    println!("Hello, world!");

    let par = parser::ProgramParser::new();
    // let src = "print (3 + 4) * 5;";
    // let src = "x = 3; y = x * 2; x = x + 1; y = x + y; print y;";
    let src = "x = 10; y = 1; while x > 0 do y = y * x; x = x - 1; end print y;";
    // let src = "x = 5; y = 3; if x > y then print 2; else print 4; end";
    let p = par.parse(src).expect("valid syntax");

    let mut com = Compiler::new();
    com.compile_program(&p);

    let code = com.output();
    println!("{:?}", code);

    let mut vm = VM::new(code);

    vm.execute();
    vm.dump_state();
}
