pub mod expr_parser;
use crate::{parser::expr_parser::generate_expr_ast, tokenizer::Token};
use expr_parser::{BinOpPrecedenceLevel, Expr};

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Return(Expr),
    Declare(String, Option<Expr>),
    CompoundStmt(Vec<Statement>),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    Expr(Expr),
}

pub struct TokenCursor {
    contents: Vec<Token>,
    index: usize,
}

impl TokenCursor {
    pub fn new(contents: Vec<Token>) -> Self {
        TokenCursor { contents, index: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.contents.get(self.index)
    }
    fn peek_nth(&self, n: usize) -> Option<&Token> {
        // peek_nth(1) is equivalent to peek()
        self.contents.get(self.index + n - 1)
    }

    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.contents.get(self.index - 1)
    }
}

pub fn generate_program_ast(tokens: Vec<Token>) -> Program {
    let mut tokens = TokenCursor::new(tokens);
    let f = generate_function_ast(&mut tokens);
    assert_eq!(tokens.next(), None);
    Program { function: f }
}

fn generate_function_ast(tokens: &mut TokenCursor) -> Function {
    let function_name;

    assert_eq!(tokens.next(), Some(&Token::IntT));

    if let Some(Token::Identifier { val }) = tokens.next() {
        function_name = val.clone();
    } else {
        panic!();
    }

    assert_eq!(tokens.next(), Some(&Token::OpenParen));
    assert_eq!(tokens.next(), Some(&Token::CloseParen));

    let body = generate_compound_stmt_ast(tokens);

    Function {
        name: function_name,
        body,
    }
}

fn generate_compound_stmt_ast(tokens: &mut TokenCursor) -> Vec<Statement> {
    assert_eq!(tokens.next(), Some(&Token::OpenBrace));
    let mut statements = Vec::new();

    while tokens.peek().is_some() && *tokens.peek().unwrap() != Token::CloseBrace {
        statements.push(generate_statement_ast(tokens));
    }

    assert_eq!(tokens.next(), Some(&Token::CloseBrace));
    return statements;
}

fn generate_statement_ast(tokens: &mut TokenCursor) -> Statement {
    let expr;

    match tokens.peek() {
        Some(Token::Return) => {
            tokens.next(); // consume the "return"

            expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            assert_eq!(tokens.next(), Some(&Token::Semicolon));
            return Statement::Return(expr);
        }
        Some(Token::IntT) => {
            tokens.next();
            let decl_identifier;
            let mut optional_expr = None;
            if let Some(Token::Identifier { val }) = tokens.next() {
                decl_identifier = val.clone();
            } else {
                panic!();
            }

            if tokens.peek() == Some(&Token::AssignmentEquals) {
                tokens.next();
                optional_expr = Some(generate_expr_ast(
                    tokens,
                    BinOpPrecedenceLevel::lowest_level(),
                ))
            }
            assert_eq!(tokens.next(), Some(&Token::Semicolon));
            return Statement::Declare(decl_identifier, optional_expr);
        }
        Some(Token::OpenBrace) => {
            let compound_stmt = generate_compound_stmt_ast(tokens);
            // note that a compound statement does not end in a semicolon, so there is no need here to consume a semicolon.
            return Statement::CompoundStmt(compound_stmt);
        }
        Some(Token::If) => {
            // consume the "if"
            tokens.next();
            assert_eq!(tokens.next(), Some(&Token::OpenParen));
            let conditional_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            assert_eq!(tokens.next(), Some(&Token::CloseParen));
            let taken_branch_stmt = generate_statement_ast(tokens);
            let mut not_taken_branch_stmt = None;
            if tokens.peek() == Some(&Token::Else) {
                // consume the "else"
                tokens.next();
                not_taken_branch_stmt = Some(Box::new(generate_statement_ast(tokens)));
            }

            return Statement::If(
                conditional_expr,
                Box::new(taken_branch_stmt),
                not_taken_branch_stmt,
            );
        }
        _ => {
            expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            assert_eq!(tokens.next(), Some(&Token::Semicolon));
            return Statement::Expr(expr);
        }
    }
}
