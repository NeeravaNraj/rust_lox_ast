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

use super::expr::TernaryExpr;

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

    pub fn parse(&mut self) -> Result<Box<Expr>, LoxError> {
        self.expression()
    }

    fn consume(&mut self, token: TokenType, error: LoxErrorsTypes) -> Result<&Token, LoxError> {
        if self.check(token) {
            return Ok(self.advance());
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
            return Ok(Box::new(Expr::Unary(UnaryExpr::new(operator.dup(), self.unary()?))));
        }

        self.primary()
    }

    fn factor(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.unary()?;

        while self.is_match(
            vec![TokenType::Slash, TokenType::Star]
        ) {
            let operator = self.previous();    
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), self.unary()?)));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.factor()?;

        while self.is_match(
            vec![TokenType::Plus, TokenType::Minus]
        ) {
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), self.factor()?)));
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
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), self.term()?)));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.comparison()?;

        while self.is_match(vec![TokenType::BangEqual, TokenType::Equals]) {
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), self.comparison()?)));
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Box<Expr>, LoxError> {
        let expr = self.equality()?;

        if self.match_single_token(TokenType::QuestionMark) {
            let operator = self.tokens.get(self.curr - 1).unwrap();
            let middle = self.expression()?;
            if self.match_single_token(TokenType::Colon) {
                let colon = self.previous();
                return Ok(Box::new(Expr::Ternary(TernaryExpr::new(expr, operator.dup(), middle, colon.dup(), self.expression()?))));
            }
            return Err(self.error_handler.error(self.previous(), LoxErrorsTypes::SyntaxError("Incomplete ternary operation,".to_string())));
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Box<Expr>, LoxError> {
        self.ternary()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.curr).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.curr - 1).unwrap()
    }

    fn advance(&mut self) -> &Token {
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
        }

        match self.peek().token_type {
            TokenType::Class => return,
            TokenType::DefFn => return,
            TokenType::Let => return,
            TokenType::For => return,
            TokenType::If => return,
            TokenType::Else => return,
            TokenType::Return => return,
            TokenType::Print => return,
            TokenType::While => return,
            _ => ()
        };

        self.advance();
    }
}
