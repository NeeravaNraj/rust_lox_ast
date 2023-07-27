use std::{collections::HashMap, process, rc::Rc};

use crate::{
    error::{loxerrorhandler::LoxErrorHandler, LoxErrorsTypes, LoxResult},
    lexer::literal::Literal,
    lexer::token::Token,
    lexer::tokentype::TokenType,
    loxlib::{number::loxnumber::LoxNumber, string::loxstring::LoxString},
};

pub struct Scanner<'a> {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    curr: usize,
    error_handler: &'a LoxErrorHandler,
    keywords: HashMap<String, TokenType>,
    line: i32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str, err_handler: &'a LoxErrorHandler) -> Self {
        let mut keywords: HashMap<String, TokenType> = HashMap::new();
        Scanner::load_keywords(&mut keywords);
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            curr: 0,
            error_handler: err_handler,
            keywords,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxResult> {
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
            if *ch != expected {
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
            self.error_handler.simple_error(
                self.line - 1,
                LoxErrorsTypes::Lexer(
                    "Something went in \"lexer::Scanner::peek()\", exiting".to_string(),
                ),
            );
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

    fn string(&mut self) -> Result<(), LoxResult> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(self.error_handler.simple_error(
                self.line - 1,
                LoxErrorsTypes::Syntax("String was not terminated".to_string()),
            ));
        }

        self.advance();

        // TODO: Handle escapes sequences
        let value: String = self.source[(self.start + 1)..(self.curr - 1)]
            .iter()
            .collect();
        self.add_literal(
            TokenType::String,
            Some(Literal::Str(Rc::new(LoxString::new(value)))),
        );
        Ok(())
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

        let value: String = self.source[self.start..self.curr].iter().collect();
        self.add_literal(
            TokenType::Number,
            Some(Literal::Number(Rc::new(LoxNumber::new(
                value.parse::<f64>().unwrap(),
            )))),
        );
    }

    fn identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text: String = self.source[self.start..self.curr].iter().collect();
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

    fn block_comment(&mut self) -> Result<(), LoxResult> {
        loop {
            match self.peek() {
                '/' => {
                    self.advance();
                    if self.is_match('*') {
                        self.block_comment()?;
                    }
                }
                '*' => {
                    self.advance();
                    if self.is_match('/') {
                        return Ok(());
                    }
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '\0' => {
                    return Err(self.error_handler.simple_error(
                        self.line - 1,
                        LoxErrorsTypes::Syntax("Comment block was not terminated".to_string()),
                    ))
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn scan_token(&mut self) -> Result<(), LoxResult> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => {
                let token = if self.is_match('=') {
                    TokenType::MinusEqual
                } else if self.is_match('-') {
                    TokenType::MinusMinus
                } else {
                    TokenType::Minus
                };
                self.add_token(token);
            }
            '+' => {
                let token = if self.is_match('=') {
                    TokenType::PlusEqual
                } else if self.is_match('+') {
                    TokenType::PlusPlus
                } else {
                    TokenType::Plus
                };
                self.add_token(token);
            }
            '%' => {
                let token = if self.is_match('=') {
                    TokenType::ModEqual
                } else {
                    TokenType::Modulus
                };
                self.add_token(token);
            }
            '?' => self.add_token(TokenType::QuestionMark),
            ':' => self.add_token(TokenType::Colon),
            '*' => {
                let token = if self.is_match('=') {
                    TokenType::StarEqual
                } else {
                    TokenType::Star
                };
                self.add_token(token);
            }
            ';' => self.add_token(TokenType::Semicolon),
            '!' => {
                let token = if self.is_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.is_match('=') {
                    TokenType::Equals
                } else {
                    TokenType::Assign
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.is_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.is_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token);
            }
            '/' => {
                if self.is_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.is_match('*') {
                    self.block_comment()?;
                } else if self.is_match('=') {
                    self.add_token(TokenType::SlashEqual);
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            _ if c.is_ascii_alphabetic() => self.identifier(),
            '0'..='9' => self.number(),
            '"' => self.string()?,
            ' ' => (),
            '\r' => (),
            '\t' => (),
            '\n' => self.line += 1,
            _ => {
                return Err(self.error_handler.simple_error(
                    self.line,
                    LoxErrorsTypes::Syntax(format!("Unknown character {}", c)),
                ));
            }
        }

        Ok(())
    }

    fn add_token(&mut self, token: TokenType) {
        self.add_literal(token, None);
    }

    fn add_literal(&mut self, token: TokenType, literal: Option<Literal>) {
        let text: String = self.source[self.start..self.curr].iter().collect();
        self.tokens
            .push(Token::new(token, text, literal, self.line))
    }

    fn is_alpha(ch: char) -> bool {
        ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ch == '_'
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
        hmap.insert(String::from("return"), TokenType::Return);
        hmap.insert(String::from("this"), TokenType::This);
        hmap.insert(String::from("true"), TokenType::True);
        hmap.insert(String::from("false"), TokenType::False);
        hmap.insert(String::from("break"), TokenType::Break);
        hmap.insert(String::from("continue"), TokenType::Continue);
        hmap.insert(String::from("fn"), TokenType::DefFn);
        hmap.insert(String::from("class"), TokenType::Class);
        hmap.insert(String::from("lm"), TokenType::DefLambda);
        hmap.insert(String::from("elif"), TokenType::Elif);
        hmap.insert(String::from("static"), TokenType::Static);
        hmap.insert(String::from("public"), TokenType::Public);
        hmap.insert(String::from("private"), TokenType::Private);
    }

    fn get_literal_type(&self, token: &TokenType) -> Literal {
        match *token {
            TokenType::None => Literal::None,
            TokenType::True => Literal::Bool(true),
            TokenType::False => Literal::Bool(false),
            _ => Literal::LiteralNone,
        }
    }

    fn is_literal_type(&self, token: &TokenType) -> bool {
        matches!(*token, TokenType::True | TokenType::False | TokenType::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_char_tokens() {
        let src = "(){}[],.-+*/;?:%";
        let e_handler = LoxErrorHandler::new();
        let mut s = Scanner::new(src, &e_handler);
        let expected = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::LeftBracket,
            TokenType::RightBracket,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Star,
            TokenType::Slash,
            TokenType::Semicolon,
            TokenType::QuestionMark,
            TokenType::Colon,
            TokenType::Modulus,
            TokenType::EOF,
        ];

        let lexemes = vec![
            String::from("("),
            String::from(")"),
            String::from("{"),
            String::from("}"),
            String::from("["),
            String::from("]"),
            String::from(","),
            String::from("."),
            String::from("-"),
            String::from("+"),
            String::from("*"),
            String::from("/"),
            String::from(";"),
            String::from("?"),
            String::from(":"),
            String::from("%"),
        ];

        match s.scan_tokens() {
            Ok(toks) => {
                assert_eq!(expected.len(), toks.len());
                for (i, val) in toks.iter().enumerate() {
                    assert_eq!(&val.token_type, expected.get(i).unwrap());
                    if let Some(lit) = lexemes.get(i) {
                        assert_eq!(&val.lexeme, lit);
                    }
                }
            }
            Err(_) => panic!("failed"),
        }
    }

    #[test]
    fn multiple_character_tokens() {
        let src = "! != += -= *= /= = == > >= < <=";
        let e_handler = LoxErrorHandler::new();
        let mut s = Scanner::new(src, &e_handler);
        let expected = vec![
            TokenType::Bang,
            TokenType::BangEqual,
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::StarEqual,
            TokenType::SlashEqual,
            TokenType::Assign,
            TokenType::Equals,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::EOF,
        ];

        let lexemes = vec![
            String::from("!"),
            String::from("!="),
            String::from("+="),
            String::from("-="),
            String::from("*="),
            String::from("/="),
            String::from("="),
            String::from("=="),
            String::from(">"),
            String::from(">="),
            String::from("<"),
            String::from("<="),
        ];
        match s.scan_tokens() {
            Ok(toks) => {
                assert_eq!(expected.len(), toks.len());
                for (i, val) in toks.iter().enumerate() {
                    assert_eq!(&val.token_type, expected.get(i).unwrap());
                    if let Some(lex) = lexemes.get(i) {
                        assert_eq!(&val.lexeme, lex);
                    }
                }
            }
            Err(_) => panic!("failed"),
        }
    }

    #[test]
    fn literal_tokens() {
        let src = "identifier \"string\" 123";
        let e_handler = LoxErrorHandler::new();
        let mut s = Scanner::new(src, &e_handler);
        let expected_token = vec![
            TokenType::Identifier,
            TokenType::String,
            TokenType::Number,
            TokenType::EOF,
        ];

        let lexemes = vec![
            String::from("identifier"),
            String::from("\"string\""),
            String::from("123"),
        ];

        match s.scan_tokens() {
            Ok(toks) => {
                assert_eq!(expected_token.len(), toks.len());
                for (i, val) in toks.iter().enumerate() {
                    assert_eq!(&val.token_type, expected_token.get(i).unwrap());
                    if let Some(lit) = &val.literal {
                        match lit {
                            Literal::Str(_) => assert_eq!(lit.unwrap_str(), "string".to_string()),
                            Literal::Number(_) => assert_eq!(lit.unwrap_number(), 123_f64),
                            _ => assert_eq!(&val.literal, &None),
                        }
                    };
                    if let Some(lexeme) = lexemes.get(i) {
                        assert_eq!(&val.lexeme, lexeme);
                    };
                }
            }
            Err(_) => panic!("failed"),
        }
    }

    #[test]
    fn keyword_tokens() {
        let src = "and class else false fn for if or return this true let none while break continue";
        let e_handler = LoxErrorHandler::new();
        let mut s = Scanner::new(src, &e_handler);
        let expected_token = vec![
            TokenType::And,
            TokenType::Class,
            TokenType::Else,
            TokenType::False,
            TokenType::DefFn,
            TokenType::For,
            TokenType::If,
            TokenType::Or,
            TokenType::Return,
            TokenType::This,
            TokenType::True,
            TokenType::Let,
            TokenType::None,
            TokenType::While,
            TokenType::Break,
            TokenType::Continue,
            TokenType::EOF,
        ];

        let lexemes = vec![
            String::from("and"),
            String::from("class"),
            String::from("else"),
            String::from("false"),
            String::from("fn"),
            String::from("for"),
            String::from("if"),
            String::from("or"),
            String::from("return"),
            String::from("this"),
            String::from("true"),
            String::from("let"),
            String::from("none"),
            String::from("while"),
            String::from("break"),
            String::from("continue"),
            String::from(""),
        ];

        match s.scan_tokens() {
            Ok(toks) => {
                assert_eq!(expected_token.len(), toks.len());
                for (i, val) in toks.iter().enumerate() {
                    assert_eq!(&val.token_type, expected_token.get(i).unwrap());
                    if let Some(lexeme) = lexemes.get(i) {
                        assert_eq!(&val.lexeme, lexeme);
                    };
                }
            }
            Err(_) => panic!("failed"),
        }
    }

    #[test]
    fn unknown_character() {
        let src = "$";
        let e_handler = LoxErrorHandler::new();
        let mut s = Scanner::new(src, &e_handler);
        let expected = LoxErrorsTypes::Syntax("Unknown character $".to_string());
        match s.scan_tokens() {
            Ok(_) => panic!("failed"),
            Err(err) => match err {
                LoxResult::Error(err) => {
                    assert_eq!(err.error_type, expected);
                }
                _ => {}
            },
        }
    }

    #[test]
    fn string_termination_err() {
        let src = "\"awdawdawdadadad";
        let e_handler = LoxErrorHandler::new();
        let mut s = Scanner::new(src, &e_handler);
        let expected = LoxErrorsTypes::Syntax("String was not terminated".to_string());
        match s.scan_tokens() {
            Ok(_) => panic!("failed"),
            Err(err) => match err {
                LoxResult::Error(err) => {
                    assert_eq!(err.error_type, expected);
                }
                _ => {}
            },
        }
    }

    #[test]
    fn comment_termination_err() {
        let src = "/* awdawdawdadadad";
        let e_handler = LoxErrorHandler::new();
        let mut s = Scanner::new(src, &e_handler);
        let expected = LoxErrorsTypes::Syntax("Comment block was not terminated".to_string());
        match s.scan_tokens() {
            Ok(_) => panic!("failed"),
            Err(err) => match err {
                LoxResult::Error(err) => {
                    assert_eq!(err.error_type, expected);
                }
                _ => {}
            },
        }
    }
}
