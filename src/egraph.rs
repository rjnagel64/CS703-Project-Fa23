



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

use egg::SymbolLang;
use egg::RecExpr;
use egg::EGraph;
use egg::Rewrite;
use egg::rewrite;
use egg::Runner;
use egg::Extractor;

pub fn demo() {
    println!("EGG");

    let mut expr = RecExpr::default();
    let a1 = expr.add(SymbolLang::leaf("a"));
    let b1 = expr.add(SymbolLang::leaf("0"));
    let foo1 = expr.add(SymbolLang::new("+", vec![a1, b1]));
    let a2 = expr.add(SymbolLang::leaf("a"));
    let b2 = expr.add(SymbolLang::leaf("b"));
    let foo2 = expr.add(SymbolLang::new("+", vec![a2, b2]));
    let bar = expr.add(SymbolLang::new("*", vec![foo1, foo2]));
    println!("this expr is {}", expr);

    // let mut egraph = EGraph::new(());
    // let a1 = egraph.add(SymbolLang::leaf("a"));
    // let b1 = egraph.add(SymbolLang::leaf("0"));
    // let foo1 = egraph.add(SymbolLang::new("+", vec![a1, b1]));
    // let a2 = egraph.add(SymbolLang::leaf("a"));
    // let b2 = egraph.add(SymbolLang::leaf("0"));
    // let foo2 = egraph.add(SymbolLang::new("+", vec![a2, b2]));
    // let bar = egraph.add(SymbolLang::new("*", vec![foo1, foo2]));
    // println!("{:?}", egraph.dump());


    let rules: &[Rewrite<SymbolLang, ()>] = &[
        rewrite!("add-0"; "(+ ?x 0)" => "?x"),
    ];

    let runner = Runner::default().with_expr(&expr).run(rules);
    let extractor = Extractor::new(&runner.egraph, egg::AstSize);
    let (best_cost, best_expr) = extractor.find_best(runner.roots[0]);
    println!("{}", best_expr);

}
