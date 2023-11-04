
pub enum Expr {
    Var(String),
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
    Assign(String, Box<Expr>),
    If(Box<Expr>, Block, Block),
    While(Box<Expr>, Block),
    Print(Box<Expr>),
}

pub struct Block(pub Vec<Stmt>);

pub struct Program(pub Block);

