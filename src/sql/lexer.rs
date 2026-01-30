use crate::util::error::{FerzError, Result};
use crate::util::types::Value;

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    Literal(Value),
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Asterisk,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Select, Insert, Update, Delete, Create, Table, Database, From, Into, Values, Where, Limit,
    Integer, Text, Primary, Key, And, Or,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s.to_uppercase().as_str() {
            "SELECT" => Some(Keyword::Select),
            "INSERT" => Some(Keyword::Insert),
            "UPDATE" => Some(Keyword::Update),
            "DELETE" => Some(Keyword::Delete),
            "CREATE" => Some(Keyword::Create),
            "TABLE" => Some(Keyword::Table),
            "DATABASE" => Some(Keyword::Database),
            "FROM" => Some(Keyword::From),
            "INTO" => Some(Keyword::Into),
            "VALUES" => Some(Keyword::Values),
            "WHERE" => Some(Keyword::Where),
            "LIMIT" => Some(Keyword::Limit),
            "INTEGER" => Some(Keyword::Integer),
            "TEXT" => Some(Keyword::Text),
            "PRIMARY" => Some(Keyword::Primary),
            "KEY" => Some(Keyword::Key),
            "AND" => Some(Keyword::And),
            "OR" => Some(Keyword::Or),
            _ => None,
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self { input: input.chars().collect(), pos: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                tokens.push(Token::Eof);
                break;
            }
            let token = self.next_token()?;
            tokens.push(token.clone());
            if token == Token::Eof { break; }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return Ok(Token::Eof); }
        let ch = self.input[self.pos];
        match ch {
            '(' => { self.pos += 1; Ok(Token::LeftParen) }
            ')' => { self.pos += 1; Ok(Token::RightParen) }
            ',' => { self.pos += 1; Ok(Token::Comma) }
            '*' => { self.pos += 1; Ok(Token::Asterisk) }
            '=' => { self.pos += 1; Ok(Token::Equal) }
            '\'' => self.read_string(),
             ch if ch.is_ascii_digit() => self.read_number(),
             ch if ch.is_alphabetic() || ch == '_' => self.read_identifier_or_keyword(),
            ';' => { self.pos += 1; Ok(Token::Semicolon) }
            _ => Err(FerzError::ParseError(format!("Unexpected char: {}", ch))),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token> {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_') {
            self.pos += 1;
        }
        let text: String = self.input[start..self.pos].iter().collect();
        if let Some(kw) = Keyword::from_str(&text) {
            Ok(Token::Keyword(kw))
        } else {
            Ok(Token::Identifier(text.to_lowercase()))
        }
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let s: String = self.input[start..self.pos].iter().collect();
        let val = s.parse::<i64>().map_err(|_| FerzError::ParseError(format!("Invalid int: {}", s)))?;
        Ok(Token::Literal(Value::Integer(val)))
    }

    fn read_string(&mut self) -> Result<Token> {
        self.pos += 1; // skip '
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != '\'' {
            self.pos += 1;
        }
        if self.pos >= self.input.len() { return Err(FerzError::ParseError("Unterminated string".to_string())); }
        let text: String = self.input[start..self.pos].iter().collect();
        self.pos += 1; // skip '
        Ok(Token::Literal(Value::Text(text)))
    }
}
