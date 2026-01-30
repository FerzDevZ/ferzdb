use crate::sql::lexer::{Lexer, Token, Keyword};
use crate::sql::ast::{Statement, ColumnDef, SelectColumn};
use crate::util::error::{FerzError, Result};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(sql: &str) -> Result<Self> {
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize()?;
        Ok(Self { tokens, pos: 0 })
    }

    pub fn parse(&mut self) -> Result<Statement> {
        let stmt = match self.peek() {
            Token::Keyword(Keyword::Select) => self.parse_select()?,
            Token::Keyword(Keyword::Insert) => self.parse_insert()?,
            Token::Keyword(Keyword::Create) => self.parse_create()?,
            _ => return Err(FerzError::ParseError("Expected SELECT, INSERT, or CREATE".to_string())),
        };
        
        // Optional semicolon
        if self.peek() == Token::Semicolon {
            self.consume();
        }
        
        Ok(stmt)
    }

    fn parse_select(&mut self) -> Result<Statement> {
        self.consume_keyword(Keyword::Select)?;
        
        let mut columns = Vec::new();
        if self.peek() == Token::Asterisk {
            self.consume();
            columns.push(SelectColumn::All);
        } else {
            // Very basic: just one column for now or implement list
            let col = self.consume_identifier()?;
            columns.push(SelectColumn::Named(col));
        }

        self.consume_keyword(Keyword::From)?;
        let from = self.consume_identifier()?;
        
        let mut where_clause = None;
        if self.peek() == Token::Keyword(Keyword::Where) {
            self.consume();
            let col_name = self.consume_identifier()?;
            self.consume_token(Token::Equal)?;
            let val = self.parse_expression_literal()?;
            
            where_clause = Some(crate::sql::ast::Expression::BinaryOp {
                left: Box::new(crate::sql::ast::Expression::Column(col_name)),
                op: crate::sql::ast::BinaryOperator::Eq,
                right: Box::new(val),
            });
        }

        Ok(Statement::Select {
            columns,
            from,
            where_clause,
            order_by: None,
            limit: None,
        })
    }

    fn parse_insert(&mut self) -> Result<Statement> {
        self.consume_keyword(Keyword::Insert)?;
        self.consume_keyword(Keyword::Into)?;
        let table = self.consume_identifier()?;
        self.consume_keyword(Keyword::Values)?;
        
        self.consume_token(Token::LeftParen)?;
        let mut values = Vec::new();
        loop {
            values.push(self.parse_expression_literal()?);
            if self.peek() == Token::Comma {
                self.consume();
            } else {
                break;
            }
        }
        self.consume_token(Token::RightParen)?;
        
        Ok(Statement::Insert { 
            table, 
            columns: None, 
            values 
        })
    }

    fn parse_expression_literal(&mut self) -> Result<crate::sql::ast::Expression> {
        match self.consume() {
            Token::Literal(v) => Ok(crate::sql::ast::Expression::Literal(v)),
            t => Err(FerzError::ParseError(format!("Expected literal, found {:?}", t))),
        }
    }

    fn parse_create(&mut self) -> Result<Statement> {
        self.consume_keyword(Keyword::Create)?;
        self.consume_keyword(Keyword::Table)?;
        let name = self.consume_identifier()?;
        
        let mut columns = Vec::new();
        if self.peek() == Token::LeftParen {
            self.consume();
            loop {
                let col_name = self.consume_identifier()?;
                let data_type = self.consume_data_type()?;
                columns.push(ColumnDef { 
                    name: col_name, 
                    data_type, 
                    primary_key: false, 
                    not_null: false, 
                });
                
                if self.peek() == Token::Comma {
                    self.consume();
                } else {
                    break;
                }
            }
            self.consume_token(Token::RightParen)?;
        }

        Ok(Statement::CreateTable { 
            name, 
            columns 
        })
    }

    fn consume_data_type(&mut self) -> Result<crate::util::types::DataType> {
        use crate::util::types::DataType;
        match self.consume() {
            Token::Keyword(Keyword::Integer) => Ok(DataType::Integer),
            Token::Keyword(Keyword::Text) => Ok(DataType::Text),
            t => Err(FerzError::ParseError(format!("Expected data type, found {:?}", t))),
        }
    }

    fn peek(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token::Eof
        }
    }

    fn consume(&mut self) -> Token {
        let t = self.peek();
        self.pos += 1;
        t
    }

    fn consume_token(&mut self, expected: Token) -> Result<()> {
        let actual = self.consume();
        if actual == expected { Ok(()) }
        else { Err(FerzError::ParseError(format!("Expected {:?}, found {:?}", expected, actual))) }
    }

    fn consume_keyword(&mut self, kw: Keyword) -> Result<()> {
        match self.consume() {
            Token::Keyword(k) if k == kw => Ok(()),
            t => Err(FerzError::ParseError(format!("Expected keyword {:?}, found {:?}", kw, t))),
        }
    }

    fn consume_identifier(&mut self) -> Result<String> {
        match self.consume() {
            Token::Identifier(id) => Ok(id),
            t => Err(FerzError::ParseError(format!("Expected identifier, found {:?}", t))),
        }
    }
}
