
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Var(pub String);

pub enum Expr {
    Var(Var),
    Num(i64),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Lt,
    Gt,
}

pub enum Stmt {
    Assign(Var, Box<Expr>),
    If(Box<Expr>, Block, Block),
    While(Box<Expr>, Block),
    Print(Box<Expr>),
    Input(Var),
}

// TODO: Replace 'input x;' statement with 'x = args(0);' expression.
// This lets me avoid mutating global state when reading input, so e-graph conversion should be
// simpler.
//
// The primary downside of this is that I can now only consume a fixed number of arguments, i.e., I
// can no longer do something like this:
//
// ```
// # Read `n` values from the command line and compute their sum.
// input n;
// acc = 0;
// while n > 0 do
//   input x;
//   acc = acc + x;
//   n = n - 1;
// end;
// print acc;
// ```
//
// However, this isn't really the kind of program I intended to write in the first place, so I
// don't really feel bad about forbidding it.

pub struct Block(pub Vec<Stmt>);

pub struct Program(pub Block);

