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

pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
    error_handler: ParseErrorHandler,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            curr: 0,
            error_handler: ParseErrorHandler::new()
        }
    }

    fn consume(&mut self, token: TokenType, error: &str) -> Result<&Token, LoxError> {
        if self.check(token) {
            return Ok(self.advance());
        }

        Err(self.error_handler.error(
                &self.peek(), LoxErrorsTypes::ParseError(error.to_string())
        ))
    }

    fn primary(&mut self) -> Result<Box<Expr>, LoxError> {
        if self.match_single_token(TokenType::False) {
            return Ok(Box::new(Expr::Literal(LiteralExpr::new(Literal::False))));
        }

        if self.match_single_token(TokenType::True) {
            return Ok(Box::new(Expr::Literal(LiteralExpr::new(Literal::True))));
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
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;
            return Ok(Box::new(Expr::Grouping(GroupingExpr::new(expr))));
        }

        Err(self.error_handler.error(&self.previous(), LoxErrorsTypes::UknownLiteral))
    }

    fn unary(&mut self) -> Result<Box<Expr>, LoxError> {
        if self.is_match(
            vec![TokenType::Bang, TokenType::Minus]
        ) {
            let right = self.unary()?;
            let operator = self.previous();
            return Ok(Box::new(Expr::Unary(UnaryExpr::new(operator.dup(), right))));
        }

        self.primary()
    }

    fn factor(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.unary()?;

        while self.is_match(
            vec![TokenType::Slash, TokenType::Star]
        ) {
            let right = self.unary()?;
            let operator = self.previous();    
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), right)));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.factor()?;

        while self.is_match(
            vec![TokenType::Plus, TokenType::Minus]
        ) {
            let right = self.factor()?;
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), right)));
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
            let right = self.term()?;
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), right)));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.comparison()?;

        while self.is_match(vec![TokenType::BangEqual, TokenType::Equals]) {
            let right = self.comparison()?;
            let operator = self.previous();
            expr = Box::new(Expr::Binary(BinaryExpr::new(expr, operator.dup(), right)));
        }
        Ok(expr)
    }

    fn expression(&mut self) -> Result<Box<Expr>, LoxError> {
        self.equality()
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
}
