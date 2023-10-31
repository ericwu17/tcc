use super::factor_parser::generate_factor_ast;
use super::TokenCursor;
use crate::errors::display::err_display;
use crate::tokenizer::{operator::Op, Token};
use crate::types::VarType;

#[derive(Debug, Clone)]
pub enum ExprEnum {
    Int(i64),
    Var(String),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    FunctionCall(String, Vec<Expr>), // Vec<Expr> contains the arguments of the function
    Deref(Box<Expr>),
    Ref(Box<Expr>),
    PostfixDec(Box<Expr>),
    PostfixInc(Box<Expr>),
    PrefixDec(Box<Expr>),
    PrefixInc(Box<Expr>),
    Sizeof(Box<Expr>),
    ArrInitExpr(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub content: ExprEnum,
    pub type_: Option<VarType>,
}

impl Expr {
    pub fn new(content: ExprEnum) -> Self {
        Expr {
            content,
            type_: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Negation,
    BitwiseComplement,
    Not,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinOp {
    Multiply,
    Divide,
    Modulus,
    Plus,
    Minus,
    GreaterThan,
    GreaterThanEq,
    LessThan,
    LessThanEq,
    Equals,
    NotEquals,
    LogicalAnd,
    LogicalOr,
    Assign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOpPrecedenceLevel {
    MulDiv,
    AddSub,
    OrderingCmp,
    EqCmp,
    LogicalAnd,
    LogicalOr,
    Assignment,
}

impl BinOpPrecedenceLevel {
    pub fn next_level(&self) -> Option<Self> {
        match self {
            BinOpPrecedenceLevel::Assignment => Some(BinOpPrecedenceLevel::LogicalOr),
            BinOpPrecedenceLevel::LogicalOr => Some(BinOpPrecedenceLevel::LogicalAnd),
            BinOpPrecedenceLevel::LogicalAnd => Some(BinOpPrecedenceLevel::EqCmp),
            BinOpPrecedenceLevel::EqCmp => Some(BinOpPrecedenceLevel::OrderingCmp),
            BinOpPrecedenceLevel::OrderingCmp => Some(BinOpPrecedenceLevel::AddSub),
            BinOpPrecedenceLevel::AddSub => Some(BinOpPrecedenceLevel::MulDiv),
            BinOpPrecedenceLevel::MulDiv => None,
        }
    }

    pub fn lowest_level() -> Self {
        BinOpPrecedenceLevel::Assignment
    }
}

pub fn generate_expr_ast(
    tokens: &mut TokenCursor,
    curr_operator_precedence: BinOpPrecedenceLevel,
) -> Expr {
    let mut expr: Expr;
    let next_operator_precedence_option = curr_operator_precedence.next_level();

    if let Some(next_operator_precedence) = next_operator_precedence_option {
        expr = generate_expr_ast(tokens, next_operator_precedence);
    } else {
        expr = generate_factor_ast(tokens);
    }

    while tokens.peek().is_some() {
        if &Token::QuestionMark == tokens.peek().unwrap()
            && curr_operator_precedence == BinOpPrecedenceLevel::lowest_level()
        {
            // handle ternary case. Note that ternaries have the lowest precedence level, so we need to check the precedence level.
            tokens.next();
            let first_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            if tokens.next() != Some(&Token::Colon) {
                err_display(
                    format!(
                        "expected colon in ternary expression, found {:?}",
                        tokens.last().unwrap()
                    ),
                    tokens.get_last_ptr(),
                )
            }

            let second_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            return Expr::new(ExprEnum::Ternary(
                Box::new(expr),
                Box::new(first_expr),
                Box::new(second_expr),
            ));
        }

        // if the next token is a binary operator that is on the current precedence level:
        if let Some(next_op) = tokens
            .peek()
            .unwrap()
            .to_binop_precedence_level(curr_operator_precedence)
        {
            let curr_token = tokens.next().unwrap().clone();
            let next_expr;

            if curr_operator_precedence == BinOpPrecedenceLevel::Assignment {
                expr = generate_assignment_expr_ast(tokens, expr, curr_token);
            } else {
                if let Some(next_operator_precedence) = next_operator_precedence_option {
                    next_expr = generate_expr_ast(tokens, next_operator_precedence);
                } else {
                    next_expr = generate_factor_ast(tokens);
                }
                expr = Expr::new(ExprEnum::BinOp(
                    next_op,
                    Box::new(expr),
                    Box::new(next_expr),
                ));
            }
        } else {
            break;
        }
    }
    return expr;
}

fn generate_assignment_expr_ast(
    tokens: &mut TokenCursor,
    lhs_expr: Expr,
    curr_token: Token,
) -> Expr {
    // ASSIGNMENT IS RIGHT ASSOCIATIVE, so we don't increment the operator precedence.
    let next_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::Assignment);
    let expr;
    match curr_token {
        Token::Op(Op::AssignmentEquals) => {
            expr = Expr::new(ExprEnum::BinOp(
                BinOp::Assign,
                Box::new(lhs_expr),
                Box::new(next_expr),
            ));
        }
        Token::Op(Op::PlusEquals) => {
            expr = Expr::new(ExprEnum::BinOp(
                BinOp::Assign,
                Box::new(lhs_expr.clone()),
                Box::new(Expr::new(ExprEnum::BinOp(
                    BinOp::Plus,
                    Box::new(lhs_expr),
                    Box::new(next_expr),
                ))),
            ));
        }
        Token::Op(Op::MinusEquals) => {
            expr = Expr::new(ExprEnum::BinOp(
                BinOp::Assign,
                Box::new(lhs_expr.clone()),
                Box::new(Expr::new(ExprEnum::BinOp(
                    BinOp::Minus,
                    Box::new(lhs_expr),
                    Box::new(next_expr),
                ))),
            ));
        }
        Token::Op(Op::MulEquals) => {
            expr = Expr::new(ExprEnum::BinOp(
                BinOp::Assign,
                Box::new(lhs_expr.clone()),
                Box::new(Expr::new(ExprEnum::BinOp(
                    BinOp::Multiply,
                    Box::new(lhs_expr),
                    Box::new(next_expr),
                ))),
            ));
        }
        Token::Op(Op::DivEquals) => {
            expr = Expr::new(ExprEnum::BinOp(
                BinOp::Assign,
                Box::new(lhs_expr.clone()),
                Box::new(Expr::new(ExprEnum::BinOp(
                    BinOp::Divide,
                    Box::new(lhs_expr),
                    Box::new(next_expr),
                ))),
            ));
        }
        Token::Op(Op::ModEquals) => {
            expr = Expr::new(ExprEnum::BinOp(
                BinOp::Modulus,
                Box::new(lhs_expr.clone()),
                Box::new(Expr::new(ExprEnum::BinOp(
                    BinOp::Plus,
                    Box::new(lhs_expr),
                    Box::new(next_expr),
                ))),
            ));
        }
        _ => unreachable!(),
    }
    return expr;
}
