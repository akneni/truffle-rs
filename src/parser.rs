use std::{collections::{HashMap, HashSet}, default, fmt::Debug};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    I64,
    U64,
    F64,
    U8,
    Bool,
    Char,
    String,
    Vec { inner: Box<DataType> },
}

impl DataType {
    fn new(dt: &str) -> Self {
        match dt {
            "int" => return DataType::I64,
            "uint" => return DataType::U64,
            "float" => return DataType::F64,
            "bool" => return DataType::Bool,
            "char" => return DataType::Char,
            "byte" => return DataType::U8,
            "string" => return DataType::String,
            _ => {
                if !dt.contains('[') {
                    panic!("No data type found for `{}`", dt);
                }
            },
        }

        let (ty, brackets) = dt.split_once("[").unwrap();
        let mut final_dt = Self::new(ty);
        for i in 0..=(brackets.len()/2) {
            final_dt = DataType::Vec { inner: Box::new(final_dt) };
        }
        final_dt
    }

    fn is_numeric(&self) -> bool {
        let num_types = [
            Self::I64,
            Self::U64,
            Self::F64,
            Self::U8,
        ];
        num_types.contains(&self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperationType {
    Add,
    Subtract,
    Div,
    Mult,
    Mod,
    GreaterThan,
    LessThan,
    GreaterThanOrEq,
    LessThanOrEq,
    Eq,
    NotEq,
}

impl OperationType {
    fn new(token: &Token) -> Result<Self> {
        match token.token_type {
            TokenType::ArithmeticOperator => {
                let op = match token.value {
                    "+" => Self::Add,
                    "-" => Self::Subtract,
                    "*" => Self::Mult,
                    "/" => Self::Div,
                    "%" => Self::Mod,
                    _ => return Err(anyhow!("faulty arithmetic operator: `{}`", token.value)),
                };
                return Ok(op);
            }
            TokenType::ComparisonOperator => {
                let op = match token.value {
                    ">" => Self::GreaterThan,
                    "<" => Self::LessThan,
                    ">=" => Self::GreaterThanOrEq,
                    "<=" => Self::LessThanOrEq,
                    "==" => Self::Eq,
                    "!=" => Self::NotEq,
                    _ => return Err(anyhow!("faulty comparison operator: `{}`", token.value)),
                };
                return Ok(op);
            }
            _ => return Err(anyhow!("Incorrect token passed to [fn OperationType::new]")),
        }
    }

    /// 255 is highest priority, 1 is the lowest
    fn get_priority(&self) -> usize {
        if [Self::Mult, Self::Div, Self::Mod].contains(&self) {
            return 10;
        }
        else if [Self::Add, Self::Subtract].contains(&self) {
            return 9;
        }
        8
    }

    /// Returns true if the operation is a arithmetic operator
    fn is_arithmetic(&self) -> bool {
        let arith = [    
            Self::Add,
            Self::Subtract,
            Self::Div,
            Self::Mult,
            Self::Mod,
        ];
        arith.contains(&self)
    }

    /// Returns true if the operation is a comparison operator
    fn is_comparison(&self) -> bool {
        let comp = [    
            Self::GreaterThan,
            Self::LessThan,
            Self::GreaterThanOrEq,
            Self::LessThanOrEq,
            Self::Eq,
            Self::NotEq
        ];
        comp.contains(&self)
    }

    fn as_str(&self) -> &'static str {
        match &self {
            Self::Add => return "+",
            Self::Subtract => return "-",
            Self::Div => return "/",
            Self::Mult => return "*",
            Self::Mod => return "%",
            Self::GreaterThan => return ">",
            Self::LessThan => return "<",
            Self::GreaterThanOrEq => return ">=",
            Self::LessThanOrEq => return "<=",
            Self::Eq => return "==",
            Self::NotEq => return "!=",
        }
    }
}

pub trait Value {
    fn dtype(&self) -> DataType;
    fn value(&self) -> String;
}

impl Value for Literal {
    fn dtype(&self) -> DataType {
        self.dtype.clone()
    }

    fn value(&self) -> String {
        self.value.clone()
    }
}
impl Value for Variable{
    fn dtype(&self) -> DataType {
        self.dtype.clone()
    }

    fn value(&self) -> String {
        self.name.clone()
    }
}
impl Value for Operation{
    fn dtype(&self) -> DataType {
        self.ret_type.clone()
    }

    fn value(&self) -> String {
        format!("({} {} {})", self.opd_1.value(), self.op.as_str(), self.opd_2.value())
    }
}

impl Debug for dyn Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}


#[derive(Debug, Clone)]
pub struct Literal {
    value: String,
    dtype: DataType,
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    dtype: DataType,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    parameters: Vec<Variable>,
    body: CodeBlock,
}

#[derive(Debug)]
pub struct CodeBlock {
    statements: Vec<AstNode>,  // A block typically contains a sequence of AST nodes
}

#[derive(Debug)]
pub struct AssignmentStatement {
    dst: Variable,
    src: Box<dyn Value>,
}

#[derive(Debug)]
pub struct Operation {
    opd_1: Box<dyn Value>,
    opd_2: Box<dyn Value>,
    op: OperationType,
    ret_type: DataType,
}

impl Operation {
    /// Modifies the return type of the Operation object based on the types of the operands and operator
    fn gen_return_t(&mut self) {
        if self.op.is_comparison() {
            self.ret_type = DataType::Bool;
        }
        else if self.opd_1.dtype() == DataType::F64 || self.opd_2.dtype() == DataType::F64 {
            assert!(self.opd_1.dtype().is_numeric());
            assert!(self.opd_2.dtype().is_numeric());
            self.ret_type = DataType::F64;
        }
        else {
            assert_eq!(self.opd_1.dtype(), self.opd_2.dtype());
            self.ret_type = self.opd_1.dtype();
        }
    }

    fn exists_inline(tokens: &[Token]) -> bool {
        let operators = [
            TokenType::ArithmeticOperator,
            TokenType::ComparisonOperator,
        ];

        let end_tokens = [
            TokenType::NewLine,
            TokenType::OpenCurlyBrace,
            TokenType::CloseCurlyBrace,
            TokenType::SemiColon,
            TokenType::Comma
        ];

        for t in tokens.iter() {
            if operators.contains(&t.token_type) {
                return true;
            } 
            else if end_tokens.contains(&t.token_type) {
                return false;
            }
        }
        false
    }

    fn extract_operation(tokens: &[Token], variable_lst: &HashMap<String, DataType>) -> (Box<dyn Value>, usize) {
        let mut op = Operation {
            opd_1: Box::new(Literal{value:"1".to_string(), dtype: DataType::I64}),
            opd_2: Box::new(Literal{value:"1".to_string(), dtype: DataType::I64}),
            op: OperationType::GreaterThan,
            ret_type: DataType::Bool,
        };

        let end_tokens = [
            TokenType::NewLine,
            TokenType::OpenCurlyBrace,
            TokenType::CloseCurlyBrace,
            TokenType::SemiColon,
            TokenType::Comma
        ];
        let mut length = 0;
        for t in tokens.iter() {
            if end_tokens.contains(&t.token_type) {
                break;
            }
            length += 1;
        }

        (Self::extract_operation_h(&tokens[..length], variable_lst), length)
    }

    /// Preconditions:
    /// - The tokens passed to it have no addition tokens past the end of the operations
    /// - There are no parenthesis in the tokens (if there are, you need to call this recursively)
    fn extract_operation_h(tokens: &[Token], variable_lst: &HashMap<String, DataType>) -> Box<dyn Value> {
        let value_tokens = [
            TokenType::FloatLiteral,
            TokenType::StringLiteral,
            TokenType::BooleanLiteral,
            TokenType::IntegerLiteral,
            TokenType::VariableName,
        ];

        if tokens.len() == 1 {
            if value_tokens.contains(&tokens[0].token_type) {
                let (val, _) = AstNode::generate_expression(&tokens[0..1], variable_lst);
                return val;
            }
            else {
                panic!("Last token left `{:?}` not a value in [fn Operation::extract_operation_h]", tokens[0]);
            }
        }

        let mut op_idx = 0;
        let mut op_priority = 0;

        for (i, t) in tokens.iter().enumerate() {
            if let Ok(op) = OperationType::new(t) {
                let p = op.get_priority();
                if p > op_priority {
                    op_idx = i;
                    op_priority = p;
                }
            }
        }

        if op_idx == 0 {
            panic!("[fn Operations::extract_operation_h] no operation found in `{:?}`", tokens);
        }

        let mut op = Operation {
            opd_1: Self::extract_operation_h(&tokens[..op_idx], variable_lst),
            opd_2: Self::extract_operation_h(&tokens[(op_idx+1)..], variable_lst),
            op: OperationType::new(&tokens[op_idx]).unwrap(),
            ret_type: DataType::Bool,
        };
        op.gen_return_t();

        Box::new(op)
    }
}


#[derive(Debug)]
pub enum AstNode {
    Variable(Variable),  
    Function(Function),  
    CodeBlock(CodeBlock),
    AssignmentStatement(AssignmentStatement),
    Operation(Operation),
}


impl AstNode {
    pub fn generate_function(s: &[Token]) -> Function {
        if !(s[0].token_type == TokenType::Keyword && s[0].value == "fn") {
            panic!("Error, token list does not start with");
        }

        assert_eq!(s[1].token_type, TokenType::FunctionName);
        let mut func = Function{
            name: s[1].value.to_string(),
            parameters: vec![],
            body: CodeBlock{statements: vec![]}
        };

        assert_eq!(s[2].token_type, TokenType::OpenParen);

        let mut i = 3;

        while s[i].token_type != TokenType::CloseParen {
            if s[i].token_type == TokenType::Comma {
                i += 1;
                continue;
            }

            assert_eq!(s[i].token_type, TokenType::DataType);
            let var_type = DataType::new(s[i].value);

            assert_eq!(s[i+1].token_type, TokenType::VariableName);
            let var_name = s[i+1].value.to_string();

            func.parameters.push(Variable {
                name: var_name,
                dtype: var_type,
            });
            i += 2;
        }

        assert_eq!(s[i+1].token_type, TokenType::OpenCurlyBrace);
        (func.body, _) = Self::generate_code_block(&s[(i+1)..]);

        func
    }

    fn generate_code_block(s: &[Token]) -> (CodeBlock, usize) {
        assert_eq!(s[0].token_type, TokenType::OpenCurlyBrace);

        let mut block = CodeBlock {statements: vec![]};

        let mut variable_lst: HashMap<String, DataType> = HashMap::new();

        let mut i = 1;
        loop {
            match s[i].token_type {
                TokenType::NewLine => {
                    i += 1;
                    continue;
                }
                TokenType::DataType => {
                    if s[i+1].token_type == TokenType::VariableName && s[i+2].token_type == TokenType::AssignmentOperator {
                        let var_type = DataType::new(s[i].value);
                        let var_name = s[i+1].value.to_string();
                        let var = Variable {
                            name: var_name.clone(),
                            dtype: var_type.clone(),
                        };
                        
                        variable_lst.insert(var_name, var_type);
                        

                        let (val, num_tokens) = Self::generate_expression(&s[i+3..], &variable_lst);

                        let assignment = AssignmentStatement {
                            dst: var,
                            src: val,
                        };
                        block.statements.push(AstNode::AssignmentStatement(assignment));
                        i += 3 + num_tokens;

                    }
                    else {
                        panic!("This probably shouldn't happen");
                    }
                }
                TokenType::CloseCurlyBrace => break,
                _ => {}
            }
        }

        (block, i)
    }

    fn generate_expression(s: &[Token], variable_lst: &HashMap<String, DataType>) -> (Box<dyn Value>, usize) {
        if !Operation::exists_inline(s) {
            match s[0].token_type {
                TokenType::FloatLiteral => {
                    let res = Literal {
                        value: s[0].value.to_string(),
                        dtype: DataType::F64,
                    };
                    return (Box::new(res), 1);
                }
                TokenType::IntegerLiteral => {
                    let res = Literal {
                        value: s[0].value.to_string(),
                        dtype: DataType::I64,
                    };
                    return (Box::new(res), 1);
                }
                TokenType::BooleanLiteral => {
                    let res = Literal {
                        value: s[0].value.to_string(),
                        dtype: DataType::Bool,
                    };
                    return (Box::new(res), 1);
                }
                TokenType::StringLiteral => {
                    let res = Literal {
                        value: s[0].value.to_string(),
                        dtype: DataType::Vec { inner: Box::new(DataType::U8) },
                    };
                    return (Box::new(res), 1);
                }
                TokenType::VariableName => {
                    let var_name = s[0].value.to_string();

                    let var_type = match variable_lst.get(&var_name) {
                        Some(s) => s,
                        None => panic!("Undefined variable: `{}`", var_name),
                    };
                    let var_type = var_type.clone();

                    let res = Variable {
                        name: var_name,
                        dtype: var_type,
                    };
                    return (Box::new(res), 1);
                }
                _ => panic!("Syntax error in value")
            }
        }

        Operation::extract_operation(s, variable_lst)
    }
}



pub struct Parser;
