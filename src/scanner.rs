use std::process;

use crate::{token::{Token, Literal}, tokentype::TokenType, error::LoxError};

pub struct Scanner<'a> {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    curr: usize,
    error_handler: &'a LoxError,
    line: i32
}

impl<'a> Scanner<'a> {
    pub fn new(source: &String, err_handler: &'a LoxError) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            curr: 0,
            error_handler: err_handler,
            line: 1
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.curr;
            self.scan_token();
        }

        self.tokens.push(Token::eof(self.line));
        &self.tokens
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
            eprintln!("Lexer error: Getting next character in \"peek()\".");
            process::exit(1);
        })
    }

    fn peek_next(&self) -> char {
        if self.curr + 1 >= self.source.len() {
            return '\0';
        }

        *self.source.get(self.curr + 1).unwrap_or_else(|| {
            eprintln!("Lexer error: Getting next character in \"peek()\".");
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
            self.error_handler.error(self.line, 
                String::from("Unterminated string.")
            );
            return;
        }

        self.advance();

        // TODO: Handle escapes sequences
        let value: String = self.source[(self.start+1)..(self.curr-1)]
            .iter()
            .collect();
        self.add_literal(TokenType::STRING, Some(Literal::Str(value)));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next()
            .is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value: String = self.source[self.start..self.curr]
            .iter()
            .collect();
        self.add_literal(
            TokenType::NUMBER,
            Some(Literal::Number(value.parse::<f64>().unwrap()))
        );
    }

    fn identifier(&self) {
        todo!("identifier");
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LEFT_PAREN),
            ')' => self.add_token(TokenType::RIGHT_PAREN),
            '{' => self.add_token(TokenType::LEFT_BRACE),
            '}' => self.add_token(TokenType::RIGHT_BRACE),
            ',' => self.add_token(TokenType::COMMA),
            '-' => self.add_token(TokenType::MINUS),
            '+' => self.add_token(TokenType::PLUS),
            '*' => self.add_token(TokenType::STAR),
            ';' => self.add_token(TokenType::SEMICOLON),
            '!' => {
                let token = if self.is_match('=') {
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG_EQUAL
                };
                self.add_token(token);
            },
            '=' =>  {
                let token = if self.is_match('=') {
                    TokenType::EQUALS
                } else {
                    TokenType::ASSIGN
                };
                self.add_token(token);
            },
            '<' => {
                let token = if self.is_match('='){
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                };
                self.add_token(token);
            },
            '>' => {
                let token = if self.is_match('=') {
                    TokenType::GREATER_EQUAL
                } else {
                    TokenType::GREATER
                };
                self.add_token(token);
            },
            '/' => {
                if self.is_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH);
                }
            },
            _ if c.is_ascii_alphabetic() => self.identifier(),
            '0'..='9' => self.number(),
            '"' => self.string(),
            ' ' => (),
            '\r' => (),
            '\t' => (),
            '\n' => self.line += 1,
            _ => self.error_handler.error(self.line, "Unexpected character.".to_string())
            
        }
        
    }

    fn add_token(&mut self, token: TokenType) {
        self.add_literal(token, None);
    }

    fn add_literal(&mut self, token: TokenType, literal: Option<Literal>) {
        let text: String = self.source[self.start..self.curr].iter().collect();
        self.tokens.push(Token::new(token, text, literal, self.line)) 
    }
}
