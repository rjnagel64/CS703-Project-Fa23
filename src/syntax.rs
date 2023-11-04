
pub enum Expr {
    Var(String),
    Num(i64),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

pub enum BinOp {
    Add,
    Mul,
}

pub enum Stmt {
    Assign(String, Box<Expr>),
    If(Box<Expr>, Block, Block),
    While(Box<Expr>, Block),
}

pub struct Block(Vec<Stmt>);

