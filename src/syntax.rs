
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

pub struct Block(pub Vec<Stmt>);

pub struct Program(pub Block);

