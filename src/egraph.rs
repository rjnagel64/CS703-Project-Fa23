



// attempt to optimize programs by conversion to an egraph and back.
//
// straight-line code is fairly straightforward, though side effects (via print statements) do need
// to be considered
//
// if-statements introduce the need for either join points or code duplication (maybe?)
//
// while-loops are the most problematic. Back edges, infinite unfoldings, conversion to data flow
// edges, etc.


// Consider a simple straight-line program that we want to optimize:
// 
// x = args(0);
// y = 2*x + args(1);
// print y - x - y;
// z = x + x;
// print z;
//
// We have several opportunities here:
// y - x - y ==> -x + (y - y) ==> -x
// 2*x ==> x + x
// another use of x + x
//
// So there's some opportunity for reducing, assoc/comm identities, and sharing.
// the print statements should be sequenced: print [ ] >> print [  ]
// The should have edges to appropriate previous nodes.
//
// After extraction, I'll have an expression tree.
// I'm not sure how I'll deal with sharing/re-used values.
//
// TODO: Look at egg paper and see how they do optimizations


// algorithm?
// Process statements of a loop-free program in reverse order.
// when encountering a print-statement, add its expression to a set of "roots" and sequence the
// print with an accumulator .
// given a root expression, we can construct some terms. Free variables of the expression put
// demands on values from previous statements.
// if encountering an assignment for a demanded variable, add its expression as a "root".
// if encountering an if-statement, generate an if-expression and demand each branch.
//   while-loops make this way to complicated, because definitions are hard to make unique without
//   phi-nodes.

use egg::RecExpr;
use egg::EGraph;
use egg::Rewrite;
use egg::Runner;
use egg::Extractor;
use egg::Id;
use egg::Symbol;
use egg::{rewrite, define_language};

use crate::syntax::{Block, Stmt, Expr, BinOp, Var, Program};
use crate::parser::ProgramParser;
use crate::compiler::{Compiler, VM};

use std::collections::HashMap;

define_language! {
    enum GraphExpr {
        // Numeric constants
        Num(i64),
        // Abstract symbols. IIUC these would arise from (e.g.) function parameters that don't have
        // a specific value. I don't have functions, but it doesn't hurt anything to leave this
        // constructor here.
        Symbol(Symbol),

        // Hmm. It seems I have to unpack Expr::BinOp into each of its variants, due to limits of
        // the define_language! macro.
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "<" = Lt([Id; 2]),
        ">" = Gt([Id; 2]),

        // Read the argument at position <i> from the arguments array.
        "args" = ArgRef(Id),

        // '[]' is a kind of dummy value, representing "perform no IO operations" (i.e., trivial
        // effect, pure ())
        "[]" = IOInit,
        // 'e1 >>> e2' means "perform operations from e1 and then print e2"
        ">>>" = IOSeq([Id; 2]),
    }
}

struct EGraphBuilder {
    env: HashMap<Var, Id>,
    graph: EGraph<GraphExpr, ()>,
    // The "IO Root" is intended to sequence side-effecting operations by having each new 'print'
    // statement take a reference to the current IO Root, then updating the root to point to the
    // new 'print'.
    io_root: Id,
}

impl EGraphBuilder {
    fn new() -> Self {
        let mut graph = EGraph::new(());
        let io_root = graph.add(GraphExpr::IOInit);

        EGraphBuilder {
            env: HashMap::new(),
            graph: graph,
            io_root: io_root
        }
    }

    fn expression_to_egraph(&mut self, e: &Expr) -> Id {
        match e {
            Expr::Var(x) => {
                // lookup var in map of var -> id
                *self.env.get(x).unwrap()
            },
            Expr::Num(i) => {
                // emit constant node, return its id
                self.graph.add(GraphExpr::Num(*i))
            },
            Expr::BinOp(op, e1, e2) => {
                let i1 = self.expression_to_egraph(e1);
                let i2 = self.expression_to_egraph(e2);
                // add binop(i1, i2) to egraph
                match op {
                    BinOp::Add => self.graph.add(GraphExpr::Add([i1, i2])),
                    BinOp::Sub => self.graph.add(GraphExpr::Sub([i1, i2])),
                    BinOp::Mul => self.graph.add(GraphExpr::Mul([i1, i2])),
                    BinOp::Lt => self.graph.add(GraphExpr::Lt([i1, i2])),
                    BinOp::Gt => self.graph.add(GraphExpr::Gt([i1, i2])),
                }
            },
            Expr::Input(e) => {
                let i = self.expression_to_egraph(e);
                self.graph.add(GraphExpr::ArgRef(i))
            },
        }
    }

    fn block_to_egraph(&mut self, block: &Block) {
        for s in &block.0 {
            match s {
                Stmt::Assign(x, e) => {
                    let id = self.expression_to_egraph(e);
                    // add x -> id to env
                    // make sure that I deal with reassignments properly.
                    // I *think* what should happen is that each assignment of a variable creates a
                    // new definition, analogous to SSA.
                    //
                    // The tricky part is how I merge things back together. I don't know how to
                    // represent a phi-function in an egraph
                    self.env.insert(x.clone(), id);

                    // Actually, what's GraphExpr::Symbol for? opaque variables that don't have a
                    // know definition? (e.g., function parameters or user input?)
                },
                Stmt::Print(e) => {
                    let id = self.expression_to_egraph(e);
                    self.io_root = self.graph.add(GraphExpr::IOSeq([self.io_root, id]));
                },
                _ => {
                    unimplemented!("complex statements to egraph")
                },
            }
        }
    }

    fn program_to_egraph(&mut self, prog: &Program) {
        self.block_to_egraph(&prog.0)
    }

}

// Cyclic expressions: use EGraph::union(i, j) to equate i and j, creating a cycle?

fn extract_program(exprs: &RecExpr<GraphExpr>) -> Block {
    // Aha! It is actually a graph with sharing! The duplicate expressions were just a result of
    // pretty-printing!
    let expr_slice = exprs.as_ref();
    // println!("{:?}", &expr_slice);

    // Okay, so I want to preserve sharing.
    // Thus, if there is more than one use of a particular node, I want to save that node in a
    // variable somehow. (Unless that variable is just a constant, because I can rematerialize that
    // constant easily)

    let mut num_uses = HashMap::new();
    let mut count = |x: &Id| {
        *num_uses.entry(x.clone()).or_insert(0) += 1;
    };
    for ge in expr_slice {
        match ge {
            GraphExpr::Add([x, y]) => { count(x); count(y); },
            GraphExpr::Mul([x, y]) => { count(x); count(y); },
            GraphExpr::Sub([x, y]) => { count(x); count(y); },
            GraphExpr::Lt([x, y]) => { count(x); count(y); },
            GraphExpr::Gt([x, y]) => { count(x); count(y); },
            GraphExpr::ArgRef(x) => { count(x); },
            GraphExpr::IOSeq([x, y]) => { count(x); count(y); },
            _ => {}, // Num, Symbol, IOInit
        }
    }
    // println!("{:?}", num_uses);


    let mut builder = ExprBuilder::new(num_uses);
    for (i, ge) in expr_slice.iter().enumerate() {
        let exp = match ge {
            GraphExpr::Add([x, y]) => {
                let ex = builder.get_exp(x);
                let ey = builder.get_exp(y);
                Expr::BinOp(BinOp::Add, Box::new(ex), Box::new(ey))
            },
            GraphExpr::Mul([x, y]) => {
                let ex = builder.get_exp(x);
                let ey = builder.get_exp(y);
                Expr::BinOp(BinOp::Mul, Box::new(ex), Box::new(ey))
            },
            GraphExpr::Num(n) => {
                Expr::Num(*n)
            },
            GraphExpr::ArgRef(x) => {
                let ex = builder.get_exp(x);
                Expr::Input(Box::new(ex))
            },
            GraphExpr::IOInit => {
                Expr::Num(0) // just a dummy value. not actually used for anything other than being
                             // consumed by IOSeq
            },
            GraphExpr::IOSeq([x, y]) => {
                let _ex = builder.get_exp(x); // this value is a dummy 0. discard it.
                let ey = builder.get_exp(y);
                builder.print(ey);
                Expr::Num(0) // just a dummy value
            },
            _ => unimplemented!("lol"),
        };

        let id = Id::from(i);
        builder.add_exp(id, exp);
    }

    Block(builder.stmts)
}

struct ExprBuilder {
    // TODO: Avoid assigning constants to local variables. They can just be duplicated at their use
    // sites, because a constant/literal in the program is cheaper than a locals slot+retrieval.
    vars: HashMap<Id, Var>,
    exps: HashMap<Id, Expr>,
    stmts: Vec<Stmt>,
}

impl ExprBuilder {
    fn new(num_uses: HashMap<Id, usize>) -> Self {
        let mut vars = HashMap::new();
        for (k, v) in num_uses.into_iter() {
            if v > 1 {
                vars.insert(k, Var(format!("x{}", k)));
            }
        }
        ExprBuilder { vars, exps: HashMap::new(), stmts: Vec::new() }
    }

    fn get_exp(&mut self, x: &Id) -> Expr {
        match self.vars.get(x) {
            None => self.exps.remove(x).unwrap(),
            Some(v) => Expr::Var(v.clone()),
        }
    }

    fn add_exp(&mut self, x: Id, e: Expr) {
        match self.vars.get(&x) {
            Some(v) => {
                self.stmts.push(Stmt::Assign(v.clone(), Box::new(e)))
            },
            None => {
                self.exps.insert(x, e);
            },
        }
    }

    fn print(&mut self, e: Expr) {
        self.stmts.push(Stmt::Print(Box::new(e)));
    }
}

fn optimize(prog: &Program) -> Program {
    let mut com = EGraphBuilder::new();
    com.program_to_egraph(&prog);

    let rules: &[Rewrite<GraphExpr, ()>] = &[
        rewrite!("add-comm"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rewrite!("mult-comm"; "(* ?x ?y)" => "(* ?y ?x)"),
        rewrite!("add-0"; "(+ ?x 0)" => "?x"),
        rewrite!("mul-1"; "(* ?x 1)" => "?x"),
        rewrite!("sub-self"; "(- ?x ?x)" => "0"),
        rewrite!("mult-dist-add"; "(* ?a (+ ?b ?c))" => "(+ (* ?a ?b) (* ?a ?c))"),
    ];

    let program_root = com.io_root;
    let runner = Runner::default().with_egraph(com.graph).run(rules);
    let extractor = Extractor::new(&runner.egraph, egg::AstSize);
    let (_best_cost, best_expr) = extractor.find_best(program_root);
    // println!("{}", best_expr);

    let new_block = extract_program(&best_expr);
    let new_prog = Program(new_block);

    new_prog
}

fn execute(prog: &Program, args: Vec<i64>) {
    let mut com = Compiler::new();
    com.compile_program(&prog);

    let code = com.output();
    println!("--- Compiled bytecode: ---");
    println!("{:?}", code);

    let mut vm = VM::new(code, args);

    println!("--- Results: ---");
    vm.execute();
    vm.dump_state();
}

fn run_program(src: &str, args: Vec<i64>) {
    let parser = ProgramParser::new();

    println!("Original program:");
    let prog = parser.parse(src).expect("valid syntax");
    execute(&prog, args.clone());

    println!("");
    println!("Optimized program:");
    let new_prog = optimize(&prog);
    execute(&new_prog, args.clone());
}

pub fn demo() {
    // println!("EGG");

    let src_filename = "demo.prog";
    let src = std::fs::read_to_string(src_filename).expect("src file does not exist");
    run_program(&src, vec![3]);

    // let mut expr = RecExpr::default();
    // let a1 = expr.add(SymbolLang::leaf("a"));
    // let b1 = expr.add(SymbolLang::leaf("0"));
    // let foo1 = expr.add(SymbolLang::new("+", vec![a1, b1]));
    // let a2 = expr.add(SymbolLang::leaf("0"));
    // let b2 = expr.add(SymbolLang::leaf("b"));
    // let foo2 = expr.add(SymbolLang::new("+", vec![a2, b2]));
    // let bar = expr.add(SymbolLang::new("*", vec![foo1, foo2]));
    // println!("this expr is {}", expr);
    //
    // // let mut egraph = EGraph::new(());
    // // let a1 = egraph.add(SymbolLang::leaf("a"));
    // // let b1 = egraph.add(SymbolLang::leaf("0"));
    // // let foo1 = egraph.add(SymbolLang::new("+", vec![a1, b1]));
    // // let a2 = egraph.add(SymbolLang::leaf("a"));
    // // let b2 = egraph.add(SymbolLang::leaf("0"));
    // // let foo2 = egraph.add(SymbolLang::new("+", vec![a2, b2]));
    // // let bar = egraph.add(SymbolLang::new("*", vec![foo1, foo2]));
    // // println!("{:?}", egraph.dump());
    //
    //
    // let rules: &[Rewrite<SymbolLang, ()>] = &[
    //     rewrite!("add-comm"; "(+ ?x ?y)" => "(+ ?y ?x)"),
    //     rewrite!("add-0"; "(+ ?x 0)" => "?x"),
    // ];
    //
    // // let runner = Runner::default().with_egraph(egraph).run(rules);
    // let runner = Runner::default().with_expr(&expr).run(rules);
    // let extractor = Extractor::new(&runner.egraph, egg::AstSize);
    // let (_best_cost, best_expr) = extractor.find_best(runner.roots[0]);
    // println!("{}", best_expr);

    // hmm. translation of straight-line to egraph:
    // maintain map of Var -> Id
    // visit assignment means create expression for RHS, add LHS -> that Id to env
    // translation of var: lookup var in env
    //
    // multiple assignments: should probably gen fresh var (avoid shadowing/convert to SSA)
    // This is easy for straight-line code
    // This involves inserting phis after an if-stmt somehow

}
