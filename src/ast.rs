#[derive(Debug, Clone)]
pub enum Expr {
    StringLiteral(String),
    Call { callee: String, args: Vec<Expr> },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
