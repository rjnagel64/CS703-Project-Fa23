
use std::collections::HashMap;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser);
mod compiler;
mod syntax;
mod egraph;

use parser::ProgramParser;
use syntax::{Var, Expr, BinOp, Stmt, Block, Program};

use compiler::{Compiler, VM};


fn run_program(src: &str, args: Vec<i64>) {
    let parser = ProgramParser::new();
    let p = parser.parse(src).expect("valid syntax");

    let mut com = Compiler::new();
    com.compile_program(&p);

    let code = com.output();
    println!("--- Compiled bytecode: ---");
    println!("{:?}", code);

    let mut vm = VM::new(code, args);

    println!("--- Results: ---");
    vm.execute();
    vm.dump_state();
}

fn main() {
    // let src = "print (3 + 4) * 5;";
    // let src = "x = 3; y = x * 2; x = x + 1; y = x + y; print y;";
    // let src = "x = 5; y = 3; if x > y then print 2; else print 4; end";

    let mut args = std::env::args();
    args.next().unwrap(); // skip argv[0]
    let src_filename = args.next().expect("a filename on the command line");
    if src_filename == "TEST" {
        egraph::demo();
        return;
    }
    let src = std::fs::read_to_string(src_filename).expect("file should exist");

    let arg_vals = args.map(|n| n.parse::<i64>().unwrap()).collect();

    run_program(&src, arg_vals);
}
