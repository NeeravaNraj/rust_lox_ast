use crate::{
    lexer::token::{Token, Literal},
    lexer::tokentype::TokenType,
    parser::expr::{Expr, BinaryExpr, UnaryExpr, LiteralExpr, GroupingExpr},
    errors::{
        LoxErrorsTypes,
        LoxError,
        ParseError::ParseErrorHandler
    },
};

use super::{
    expr::*,
    stmt::*
};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    curr: usize,
    error_handler: ParseErrorHandler,
}

impl<'a> Parser<'a>{
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens,
            curr: 0,
            error_handler: ParseErrorHandler::new()
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>, LoxError> {
        let mut  statments: Vec<Box<Stmt>> = Vec::new();
        while !self.is_at_end() {
            statments.push(self.declaration()?);
        }

        Ok(statments)
    }

    fn var_declaration(&mut self) -> Result<Box<Stmt>, LoxError> {
        let name = self.consume(
            TokenType::Identifier,
            LoxErrorsTypes::SyntaxError("Expected name for identifier".to_string())
        )?;

        let initializer = if self.is_match(vec![TokenType::Assign]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, LoxErrorsTypes::SyntaxError("Expect ';' after".to_string()))?;

        Ok(Box::new(Stmt::Let(LetStmt::new(name, initializer))))
    }
    
    fn declaration(&mut self) -> Result<Box<Stmt>, LoxError> {
        let result = if self.match_single_token(TokenType::Let) {
            self.var_declaration()
        } else {
            self.statement()
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn block_stmt(&mut self) -> Result<Vec<Box<Stmt>>, LoxError> {
        let mut stmts: Vec<Box<Stmt>> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, LoxErrorsTypes::ParseError("Expected '}' after block".to_string()))?;
        Ok(stmts)
    }

    fn print_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, LoxErrorsTypes::SyntaxError("Expected ';' after".to_string()))?;
        Ok(Box::new(Stmt::Print(PrintStmt::new(expr))))
    }

    fn expr_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, LoxErrorsTypes::SyntaxError("Expected ';' after".to_string()))?;
        Ok(Box::new(Stmt::Expression(ExpressionStmt::new(expr))))
    }

    fn statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        if self.match_single_token(TokenType::Print) {
            return self.print_statement();
        } 

        if self.match_single_token(TokenType::LeftBrace) {
            return Ok(Box::new(Stmt::Block(BlockStmt::new(self.block_stmt()?))));
        }
        self.expr_statement()
    }

    fn consume(&mut self, token: TokenType, error: LoxErrorsTypes) -> Result<Token, LoxError> {
        if self.check(token) {
            return Ok(self.advance().dup());
        }

        Err(self.error_handler.error(
            &self.previous(), error
        ))
    }

    fn primary(&mut self) -> Result<Box<Expr>, LoxError> {
        if self.match_single_token(TokenType::False) {
            return Ok(Box::new(Expr::Literal(LiteralExpr::new(Literal::Bool(false)))));
        }

        if self.match_single_token(TokenType::True) {
            return Ok(Box::new(Expr::Literal(LiteralExpr::new(Literal::Bool(true)))));
        }

        if self.match_single_token(TokenType::None) {
            return Ok(Box::new(Expr::Literal(LiteralExpr::new(Literal::None))));
        }

        if self.match_single_token(TokenType::Identifier) {
            return Ok(Box::new(Expr::Variable(VariableExpr::new(self.previous()))))
        }

        if self.is_match(vec![TokenType::Number, TokenType::String]) {
            match self.previous().literal.as_ref().unwrap() {
                Literal::Number(literal) => return Ok(
                    Box::new(
                        Expr::Literal(LiteralExpr::new(Literal::Number(*literal)))
                    )),
                Literal::Str(literal) => return Ok(
                    Box::new(
                        Expr::Literal(LiteralExpr::new(Literal::Str(literal.to_string())))
                )),
                _ => {}
            }
        }

        if self.match_single_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, LoxErrorsTypes::SyntaxError("Expected ')' after expression, at".to_string()))?;
            return Ok(Box::new(Expr::Grouping(GroupingExpr::new(expr))));
        }

        if self.curr == 0 {
            return Err(self.error_handler.error(
                self.peek(), 
                LoxErrorsTypes::SyntaxError(
                    format!("leading '{}' is not supported", self.peek().lexeme)
                )
            ));
        }

        Err(self.error_handler.error(&self.previous(), LoxErrorsTypes::SyntaxError("Expected expression after".to_string())))
    }

    fn unary(&mut self) -> Result<Box<Expr>, LoxError> {
        if self.is_match(
            vec![TokenType::Bang, TokenType::Minus]
        ) {
            let operator = self.previous();
            return Ok(Box::new(Expr::Unary(UnaryExpr::new(operator, self.unary()?))));
        }

        self.primary()
    }

    fn factor(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.unary()?;

        while self.is_match(
            vec![TokenType::Slash, TokenType::Star]
        ) {
            let operator = self.previous();    
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator, self.unary()?)));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.factor()?;

        while self.is_match(
            vec![TokenType::Plus, TokenType::Minus]
        ) {
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator, self.factor()?)));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.term()?;

        while self.is_match(vec![
            TokenType::Greater, 
            TokenType::GreaterEqual, 
            TokenType::Less, 
            TokenType::LessEqual]
        ) {
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator, self.term()?)));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.comparison()?;

        while self.is_match(vec![TokenType::BangEqual, TokenType::Equals]) {
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator, self.comparison()?)));
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Box<Expr>, LoxError> {
        let expr = self.equality()?;

        if self.match_single_token(TokenType::QuestionMark) {
            let operator = self.tokens.get(self.curr - 1).unwrap().dup();
            let middle = self.expression()?;
            if self.match_single_token(TokenType::Colon) {
                let colon = self.previous();
                return Ok(Box::new(Expr::Ternary(TernaryExpr::new(expr, operator, middle, colon, self.expression()?))));
            }
            return Err(self.error_handler.error(&self.previous(), LoxErrorsTypes::SyntaxError("Incomplete ternary operation,".to_string())));
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Box<Expr>, LoxError> {
        let expr = self.ternary()?;

        if self.match_single_token(TokenType::Assign) {
            let token = self.previous();
            let value = self.assignment()?;
            
            match *expr {
                Expr::Variable(var) => {
                    let name = var.name; 
                    return Ok(Box::new(Expr::Assign(AssignExpr::new(name, value))));
                },
                _ => {
                    return Err(
                        self.error_handler.error(
                            &token, 
                            LoxErrorsTypes::ParseError("Invalid assignment target".to_string())
                        )
                    );
                }
            }
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Box<Expr>, LoxError> {
        self.assignment()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.curr).unwrap()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.curr - 1).unwrap().dup()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.curr += 1;
        }
        self.previous()
    }

    fn check(&self, token: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token 
    }

    fn match_single_token(&mut self, token: TokenType) -> bool {
        if self.check(token) {
            self.advance();
            return true;
        }
        false
    }

    fn is_match(&mut self, toks: Vec<TokenType>) -> bool {
        for token in toks {
            if self.check(token) {
                self.advance();
                return true;
            }
        } 
        false
    }

    fn synchronize(&mut self,) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class 
                    | TokenType::DefFn
                    | TokenType::Let
                    | TokenType::For
                    | TokenType::If
                    | TokenType::Else
                    | TokenType::Return
                    | TokenType::Print
                    | TokenType::While => return,
                _ => ()
            };

            self.advance();
        }
    }
}
