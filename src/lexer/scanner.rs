use std::{process, collections::HashMap};

use crate::{
    lexer::token::Token, lexer::token::Literal,
    lexer::tokentype::TokenType,
    errors::{
        LoxErrorHandler::LoxErrorHandler,
        LoxError,
        LoxErrorsTypes
    },
};

pub struct Scanner<'a> {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    curr: usize,
    error_handler: &'a LoxErrorHandler,
    keywords: HashMap<String, TokenType>,
    line: i32
}

impl<'a> Scanner<'a> {
    pub fn new(source: &String, err_handler: &'a LoxErrorHandler) -> Self {
        let mut keywords: HashMap<String, TokenType> = HashMap::new();
        Scanner::load_keywords(&mut keywords);
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            curr: 0,
            error_handler: err_handler,
            keywords,
            line: 1
        }
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.curr;
            self.scan_token()?;
        }

        self.tokens.push(Token::eof(self.line));
        Ok(&self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let result = self.source.get(self.curr).unwrap();
        self.curr += 1;
        *result
    }

    fn is_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if let Some(ch) = self.source.get(self.curr) {
            if *ch !=  expected {
                return false;
            }
        }
        
        self.curr += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        *self.source.get(self.curr).unwrap_or_else(|| {
            self.error_handler.error(self.line - 1, LoxErrorsTypes::LexerError(
                "Something went in \"lexer::Scanner::peek()\", exiting".to_string()
            ));
            process::exit(1);
        })
    }

    fn peek_next(&self) -> char {
        if self.curr + 1 >= self.source.len() {
            return '\0';
        }

        *self.source.get(self.curr + 1).unwrap_or_else(|| {
            eprintln!("Lexer error: Getting next character in \"peek()\"");
            process::exit(1);
        })
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error_handler.error(
                self.line - 1, 
                LoxErrorsTypes::SyntaxError("String was not terminated".to_string())
            );
            return;
        }

        self.advance();

        // TODO: Handle escapes sequences
        let value: String = self.source[(self.start+1)..(self.curr-1)]
            .iter()
            .collect();
        self.add_literal(TokenType::String, Some(Literal::Str(value)));
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value: String = self.source[self.start..self.curr]
            .iter()
            .collect();
        self.add_literal(
            TokenType::Number,
            Some(Literal::Number(value.parse::<f64>().unwrap()))
        );
    }

    fn identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }


        let text: String = self.source[self.start..self.curr]
            .iter()
            .collect();
        if self.keywords.contains_key(&text) {
            let token = self.keywords.get(&text).unwrap();
            if self.is_literal_type(token) {
                self.add_literal(*token, Some(self.get_literal_type(token)));
            } else {
                self.add_token(*token);
            }
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn block_comment(&mut self) {
        loop {
            match self.peek() {
                '/' => {
                    self.advance();
                    if self.is_match('*') {
                        self.block_comment();
                    }
                },
                '*' => {
                    self.advance();
                    if self.is_match('/') {
                        return;
                    }
                },
                '\n' => {
                    self.line += 1;
                    self.advance();
                },
                '\0' => {
                    self.error_handler.error(
                        self.line - 1, 
                        LoxErrorsTypes::SyntaxError("Comment block was not terminated".to_string())
                    );
                    return;
                },
                _ => {
                    self.advance();
                }
            }
        }

    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            '?' => self.add_token(TokenType::QuestionMark),
            ':' => self.add_token(TokenType::Colon),
            '*' => self.add_token(TokenType::Star),
            ';' => self.add_token(TokenType::Semicolon),
            '!' => {
                let token = if self.is_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token);
            },
            '=' =>  {
                let token = if self.is_match('=') {
                    TokenType::Equals
                } else {
                    TokenType::Assign
                };
                self.add_token(token);
            },
            '<' => {
                let token = if self.is_match('='){
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token);
            },
            '>' => {
                let token = if self.is_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token);
            },
            '/' => {
                if self.is_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.is_match('*') {
                    self.block_comment();
                } else {
                    self.add_token(TokenType::Slash);
                }
            },
            _ if c.is_ascii_alphabetic() => self.identifier(),
            '0'..='9' => self.number(),
            '"' => self.string(),
            ' ' => (),
            '\r' => (),
            '\t' => (),
            '\n' => self.line += 1,
            _ => {
                return Err(self.error_handler.error(self.line, LoxErrorsTypes::SyntaxError(format!("Unknown character {}", c))));
            }
        }

        Ok(())
    }

    fn add_token(&mut self, token: TokenType) {
        self.add_literal(token, None);
    }

    fn add_literal(&mut self, token: TokenType, literal: Option<Literal>) {
        let text: String = self.source[self.start..self.curr].iter().collect();
        self.tokens.push(Token::new(token, text, literal, self.line)) 
    }

    fn is_alpha(ch: char) -> bool {
        (ch >= 'a' && ch <= 'z') ||
        (ch >= 'A' && ch <= 'Z') ||
        ch == '_'
    }

    fn is_alphanumeric(ch: char) -> bool {
        Scanner::is_alpha(ch) || Scanner::is_digit(ch)
    }

    fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    fn load_keywords(hmap: &mut HashMap<String, TokenType>) {
        hmap.insert(String::from("and"), TokenType::And);
        hmap.insert(String::from("or"), TokenType::Or);
        hmap.insert(String::from("if"), TokenType::If);
        hmap.insert(String::from("else"), TokenType::Else);
        hmap.insert(String::from("for"), TokenType::For);
        hmap.insert(String::from("while"), TokenType::While);
        hmap.insert(String::from("none"), TokenType::None);
        hmap.insert(String::from("let"), TokenType::Let);
        hmap.insert(String::from("print"), TokenType::Print);
        hmap.insert(String::from("return"), TokenType::Return);
        hmap.insert(String::from("super"), TokenType::Super);
        hmap.insert(String::from("this"), TokenType::This);
        hmap.insert(String::from("true"), TokenType::True);
        hmap.insert(String::from("false"), TokenType::False);
    }

    fn get_literal_type(&self, token: &TokenType) -> Literal {
        match *token {
            TokenType::None => Literal::None,
            TokenType::True => Literal::Bool(true),
            TokenType::False => Literal::Bool(false),
            _ => Literal::LiteralNone
        }
    }

    fn is_literal_type(&self, token: &TokenType) -> bool {
        match *token {
            TokenType::True => true,
            TokenType::False => true,
            TokenType::None => true,
            _ => false
        }
    }
}
