use crate::{
    error::{CompilationError, ErrorCode},
    tokenizer::{Token, TokenType, tokenize},
};
use std::{collections::HashMap, fmt};

pub fn analyze(input: String) -> Vec<Instruction> {
    let mut analyzer = Analyzer::new(tokenize(input));
    analyzer.analyze().unwrap().to_vec()
}
pub struct Analyzer {
    tokens: Vec<Token>,
    offset: usize,
    instructions: Vec<Instruction>,
    current_pos: (usize, usize),
    uninitialized_vars: HashMap<String, i32>,
    vars: HashMap<String, i32>,
    consts: HashMap<String, i32>,
    next_token_index: usize,
}

impl Analyzer {
    pub fn new(tokens : Vec<Token>) -> Analyzer {
        Analyzer {
            tokens,
            offset: 0,
            instructions: Vec::new(),
            current_pos: (0, 0),
            uninitialized_vars: HashMap::new(),
            vars: HashMap::new(),
            consts: HashMap::new(),
            next_token_index: 0
        }
    }
    pub fn analyze(&mut self) -> Result<&Vec<Instruction>, CompilationError> {
        self.analyze_program().map(move |_| &self.instructions)
    }
    // <程序> ::= 'begin'<主过程>'end'
    fn analyze_program(&mut self) -> Result<(), CompilationError> {
        self.require_token(TokenType::Begin, ErrorCode::ErrNoBegin)?;
        self.analyze_main()?;
        self.require_token(TokenType::End, ErrorCode::ErrNoEnd)?;
        Ok(())
    }
    // <主过程> ::= <常量声明><变量声明><语句序列>
    fn analyze_main(&mut self) -> Result<(), CompilationError> {
        self.analyze_constant_declaration()?;
        self.analyze_variable_declaration()?;
        self.analyze_statement_sequence()?;
        Ok(())
    }
    // <常量声明> ::= {<常量声明语句>}
    // <常量声明语句> ::= 'const'<标识符>'='<常表达式>';'
    fn analyze_constant_declaration(&mut self) -> Result<(), CompilationError> {
        loop {
            match self.next_token() {
                Some(Token::Str(TokenType::Const, ..)) => (),
                Some(_) => {
                    self.unread_token();
                    return Ok(());
                }
                None => return Ok(()),
            };
            let ident = match self.next_token() {
                Some(token @Token::Str(TokenType::Identifier, ..)) => token,
                Some(_) | None => {
                    return Err(CompilationError::new_packed(
                        self.current_pos,
                        ErrorCode::ErrNeedIdentifier,
                    ))
                }
            }.clone();
            let value_string = ident.get_value_string();
            if self.is_declared(&value_string) {
                return Err(CompilationError::new_packed(
                    self.current_pos,
                    ErrorCode::ErrDuplicateDeclaration,
                ));
            }
            self.add_constant(ident);
            match self.next_token() {
                Some(Token::Str(TokenType::EqualSign, ..)) => (),
                _ => {
                    return Err(CompilationError::new_packed(
                        self.current_pos,
                        ErrorCode::ErrConstantNeedValue,
                    ))
                }
            }
            let val = self.analyze_constant_expression()?;
            match self.next_token() {
                Some(Token::Str(TokenType::Semicolon, ..)) => (),
                _ => {
                    return Err(CompilationError::new_packed(
                        self.current_pos,
                        ErrorCode::ErrNoSemicolon,
                    ))
                }
            }
            self.instructions.push(Instruction(Operation::LIT, val));
        } // loop finished
    }

    // <变量声明> ::= {<变量声明语句>}
    // <变量声明语句> ::= 'var'<标识符>['='<表达式>]';'
    fn analyze_variable_declaration(&mut self) -> Result<(), CompilationError> {
        loop {
            match self.next_token() {
                Some(Token::Str(TokenType::Var, ..)) => (),
                Some(_) => {
                    self.unread_token();
                    return Ok(());
                }
                None => return Ok(()),
            }
            let ident = self.require_token(TokenType::Identifier, ErrorCode::ErrNeedIdentifier)?.clone();
            let initialized = match self.next_token() {
                Some(Token::Str(TokenType::EqualSign, ..)) => true,
                Some(_) => {self.unread_token();false}
                None => false
            };
            if !initialized {
                self.add_uninitialized_varaible(ident);
                self.instructions.push(Instruction(Operation::LIT, 0));
                continue;
            }
            self.analyze_expression()?;
            self.require_token(TokenType::Semicolon,ErrorCode::ErrNoSemicolon)?;
            self.add_variable(ident);
        }
    }

    // <语句序列> ::= {<语句>}
    // <语句> :: = <赋值语句> | <输出语句> | <空语句>
    // <赋值语句> :: = <标识符>'='<表达式>';'
    // <输出语句> :: = 'print' '(' <表达式> ')' ';'
    // <空语句> :: = ';'
    fn analyze_statement_sequence(&mut self) -> Result<(), CompilationError> {
        loop {
            match self.next_token() {
                Some(Token::Str(TokenType::Identifier, ..)) => {self.unread_token();self.analyze_assignment_statement()?;},
                Some(Token::Str(TokenType::Print, ..)) => {self.unread_token();self.analyze_output_statement()?;},
                Some(Token::Str(TokenType::Semicolon, ..)) => (),
                None => return Ok(()),
                Some(_) => {
                    self.unread_token();
                    return Ok(());
                }
            };
        }
    }

    // <常表达式> ::= [<符号>]<无符号整数>
    fn analyze_constant_expression(&mut self) -> Result<i32, CompilationError> {
        let prefix = match self.next_token() {
            Some(Token::Str(TokenType::PlusSign, ..)) => 1,
            Some(Token::Str(TokenType::MinusSign, ..)) => -1,
            Some(_) => {self.unread_token(); 1},
            None => 1
        };
        Ok(prefix * self.require_token(TokenType::UnsignedInteger, ErrorCode::ErrIncompleteExpression)?.get_integer() as i32)
    }

    // <表达式> ::= <项>{<加法型运算符><项>}
    fn analyze_expression(&mut self) -> Result<(), CompilationError> {
        self.analyze_item()?;
        loop {
            let instr = match self.next_token() {
                Some(Token::Str(TokenType::PlusSign, ..)) => Instruction(Operation::ADD, 0),
                Some(Token::Str(TokenType::MinusSign, ..)) => Instruction(Operation::SUB, 0),
                Some(_) => {self.unread_token(); return Ok(());},
                None => return Ok(())
            };
            self.analyze_item()?;
            self.instructions.push(instr);
        }
    }

    // <赋值语句> ::= <标识符>'='<表达式>';'
    fn analyze_assignment_statement(&mut self) -> Result<(), CompilationError> {
        let name = self.require_token(TokenType::Identifier, ErrorCode::ErrNeedIdentifier)?.get_value_string().clone();
        if !self.is_declared(&name) {
            return Err(CompilationError::new_packed(self.current_pos, ErrorCode::ErrNotDeclared));
        }
        if self.is_constant(&name) {
            return Err(CompilationError::new_packed(self.current_pos, ErrorCode::ErrAssignToConstant));
        }
        self.require_token(TokenType::EqualSign, ErrorCode::ErrInvalidAssignment)?;
        self.analyze_expression()?;
        self.require_token(TokenType::Semicolon, ErrorCode::ErrNoSemicolon)?;
        self.instructions.push(Instruction(Operation::STO, self.get_index(&name).clone()));
        if !self.is_initialized_variable(&name) {
            self.make_initialized(name);
        }
        Ok(())
    }

    fn analyze_output_statement(&mut self) -> Result<(), CompilationError> {
        self.next_token(); // It is said to be escape-able
        self.require_token(TokenType::LeftBracket, ErrorCode::ErrInvalidPrint)?;
        self.analyze_expression()?;
        self.require_token(TokenType::RightBracket, ErrorCode::ErrInvalidPrint)?;
        self.require_token(TokenType::Semicolon, ErrorCode::ErrNoSemicolon)?;
        self.instructions.push(Instruction(Operation::WRT, 0));
        Ok(())
    }

    fn analyze_item(&mut self) -> Result<(), CompilationError> {
        self.analyze_factor()?;
        loop {
            let instr = match self.next_token() {
                Some(Token::Str(TokenType::MultiplicationSign, ..)) =>Instruction(Operation::MUL, 0),
                Some(Token::Str(TokenType::DivisionSign, ..)) => Instruction(Operation::DIV, 0),
                Some(_) => {self.unread_token();return Ok(())},
                None => return Ok(())
            };
            self.analyze_factor()?;
            self.instructions.push(instr);
        }
    }

    fn analyze_factor(&mut self) -> Result<(), CompilationError> {
        let prefix = match self.next_token() {
            None => {
                return Err(CompilationError::new_packed(
                    self.current_pos,
                    ErrorCode::ErrIncompleteExpression,
                ))
            }
            Some(token) => match token.get_type() {
                TokenType::PlusSign => 1,
                TokenType::MinusSign => -1,
                _ => {
                    self.unread_token();
                    1
                }
            },
        };
        match self.next_token().cloned() {
            None => return Err(CompilationError::new_packed(self.current_pos, ErrorCode::ErrIncompleteExpression)),
            Some(Token::Str(TokenType::Identifier, name, ..)) => {
                if !self.is_declared(&name) {
                    return Err(CompilationError::new_packed(self.current_pos, ErrorCode::ErrNotDeclared));
                }
                if !self.is_initialized_variable(&name) && ! self.is_constant(&name) {
                    return Err(CompilationError::new_packed(self.current_pos, ErrorCode::ErrNotInitialized));
                }
                self.instructions.push(Instruction(Operation::LOD, self.get_index(&name).clone()));
            }
            Some(Token::Integer(TokenType::UnsignedInteger, val, ..)) => {
                let value = val.clone();
                self.instructions.push(Instruction(Operation::LIT, value as i32));
            }
            Some(Token::Str(TokenType::LeftBracket, ..)) => {
                self.analyze_expression()?;
                self.require_token(TokenType::RightBracket, ErrorCode::ErrInvalidInput)?;
            }
            _ => return Err(CompilationError::new_packed(self.current_pos, ErrorCode::ErrIncompleteExpression))
        }
        if prefix == -1 {
            self.instructions.push(Instruction(Operation::SUB, 0))
        }
        Ok(())
    }

    fn require_token(&mut self, ttype :TokenType, err_code: ErrorCode) -> Result<Token, CompilationError> {
        self.next_token().cloned().filter(|t| t.get_type().eq(&ttype)).ok_or(CompilationError::new_packed(self.current_pos, err_code))
    }

    fn next_token(&mut self) -> Option<&Token> {
        if self.offset == self.tokens.len() {
            return None;
        }
        self.current_pos = self.tokens[self.offset].get_end_pos();
        self.offset += 1;
        Some(&self.tokens[self.offset])
    }

    fn unread_token(&mut self) {
        if self.offset == 0 {
            panic!("You can never unread at hajimari!");
        }
        self.offset -= 1;
        self.current_pos = self.tokens[self.offset].get_end_pos();
    }

    fn add(&mut self, token: Token, sig_type: Sigtype) {
        if !matches!(token.get_type(), TokenType::Identifier) {
            panic!("You cannot add non-identifier into sig table.");
        }
        let value_str = token.get_value_string();
        match sig_type {
            Sigtype::Const => self.consts.insert(value_str, self.next_token_index as i32),
            Sigtype::Univar => self
                .uninitialized_vars
                .insert(value_str, self.next_token_index as i32),
            Sigtype::Var => self.vars.insert(value_str, self.next_token_index as i32),
        };
        self.next_token_index += 1;
    }

    fn add_variable(&mut self, token: Token) {
        self.add(token, Sigtype::Var)
    }

    fn add_constant(&mut self, token: Token) {
        self.add(token, Sigtype::Const);
    }

    fn add_uninitialized_varaible(&mut self, token: Token) {
        self.add(token, Sigtype::Univar);
    }

    fn make_initialized(&mut self, var_name: String) {
        let item = self
            .uninitialized_vars
            .remove(&var_name)
            .expect("faile to find unini var");
        self.vars.insert(var_name, item);
    }

    fn get_index(&self, s: &str) -> &i32 {
        if let Some(u_var) = self.uninitialized_vars.get(s) {
            return u_var;
        }
        if let Some(var) = self.vars.get(s) {
            return var;
        }
        self.consts
            .get(s)
            .expect("Fail to get index from analyzer.")
    }

    fn is_declared(&self, s: &str) -> bool {
        self.is_constant(s) || self.is_initialized_variable(s) || self.is_uninitialized_variable(s)
    }

    fn is_uninitialized_variable(&self, s: &str) -> bool {
        self.uninitialized_vars.contains_key(s)
    }

    fn is_initialized_variable(&self, s: &str) -> bool {
        self.vars.contains_key(s)
    }

    fn is_constant(&self, s: &str) -> bool {
        self.consts.contains_key(s)
    }
}

enum Sigtype {
    Univar = 0,
    Var,
    Const,
}

#[derive(Clone)]
pub struct Instruction(Operation, i32);

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            0 => f.write_str("ILLIGAL!!!"),
            1..=3 => f.write_fmt(format_args!("{:?} {}", self.0, self.1)),
            _ => f.write_fmt(format_args!("{:?}", self.0)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Operation {
    ILL = 0,
    LIT,
    LOD,
    STO,
    ADD,
    SUB,
    MUL,
    DIV,
    WRT,
}
