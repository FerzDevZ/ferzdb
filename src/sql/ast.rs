//! Abstract Syntax Tree definitions for SQL statements

use crate::util::types::{DataType, Value};

/// Top-level SQL statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    CreateDatabase {
        name: String,
    },
    DropDatabase {
        name: String,
    },
    CreateTable {
        name: String,
        columns: Vec<ColumnDef>,
    },
    DropTable {
        name: String,
    },
    CreateIndex {
        name: String,
        table: String,
        column: String,
    },
    Select {
        columns: Vec<SelectColumn>,
        from: String,
        where_clause: Option<Expression>,
        order_by: Option<OrderBy>,
        limit: Option<u64>,
    },
    Insert {
        table: String,
        columns: Option<Vec<String>>,
        values: Vec<Expression>,
    },
    Update {
        table: String,
        assignments: Vec<Assignment>,
        where_clause: Option<Expression>,
    },
    Delete {
        table: String,
        where_clause: Option<Expression>,
    },
    Begin,
    Commit,
    Rollback,
}

/// Column definition in CREATE TABLE
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
    pub not_null: bool,
}

/// Select column specification
#[derive(Debug, Clone, PartialEq)]
pub enum SelectColumn {
    All,                    // *
    Named(String),          // column_name
}

/// ORDER BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    pub column: String,
    pub ascending: bool,
}

/// Assignment in UPDATE statement
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub column: String,
    pub value: Expression,
}

/// Expression (for WHERE clauses, values, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    Column(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Comparison
    Eq,         // =
    Ne,         // != or <>
    Lt,         // <
    Le,         // <=
    Gt,         // >
    Ge,         // >=
    
    // Logical
    And,
    Or,
    
    // Arithmetic (future)
    Add,
    Sub,
    Mul,
    Div,
}

impl BinaryOperator {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Eq | BinaryOperator::Ne |
            BinaryOperator::Lt | BinaryOperator::Le |
            BinaryOperator::Gt | BinaryOperator::Ge => 3,
            BinaryOperator::Add | BinaryOperator::Sub => 4,
            BinaryOperator::Mul | BinaryOperator::Div => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_create_table() {
        let stmt = Statement::CreateTable {
            name: "users".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    primary_key: true,
                    not_null: true,
                },
                ColumnDef {
                    name: "name".to_string(),
                    data_type: DataType::Text,
                    primary_key: false,
                    not_null: false,
                },
            ],
        };
        
        match stmt {
            Statement::CreateTable { name, columns } => {
                assert_eq!(name, "users");
                assert_eq!(columns.len(), 2);
            }
            _ => panic!("Expected CreateTable"),
        }
    }

    #[test]
    fn test_ast_select() {
        let stmt = Statement::Select {
            columns: vec![SelectColumn::All],
            from: "users".to_string(),
            where_clause: Some(Expression::BinaryOp {
                left: Box::new(Expression::Column("age".to_string())),
                op: BinaryOperator::Gt,
                right: Box::new(Expression::Literal(Value::Integer(18))),
            }),
            order_by: None,
            limit: Some(10),
        };
        
        match stmt {
            Statement::Select { from, limit, .. } => {
                assert_eq!(from, "users");
                assert_eq!(limit, Some(10));
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        assert!(BinaryOperator::Mul.precedence() > BinaryOperator::Add.precedence());
        assert!(BinaryOperator::And.precedence() > BinaryOperator::Or.precedence());
        assert!(BinaryOperator::Eq.precedence() > BinaryOperator::And.precedence());
    }
}
