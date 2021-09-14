use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Deref;

use crate::error::{CompilationError, ErrorCode};

pub struct Tokenizer<'a> {
    filename: &'a str,
    initialized: bool,
    lines_buffer: Vec<String>,
    ptr: (usize, usize),
}

pub fn tokenize(input: String) -> Vec<Token> {
    let mut tkz = Tokenizer::new(&input);
    tkz.get_all_tokens().unwrap()
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            filename: input,
            initialized: false,
            lines_buffer: Vec::new(),
            ptr: (0, 0)
        }
    }
    pub fn get_next_token(&mut self) -> Result<Token, CompilationError> {
        if !self.initialized {
            self.read_all();
        }
        if self.is_EOF() {
            return Err(CompilationError::new(0, 0, ErrorCode::ErrEOF));
        }
        self.next_token()
    }

    pub fn get_all_tokens(&mut self) -> Result<Vec<Token>, CompilationError> {
        let mut ret = Vec::new();
        loop {
            let token = self.get_next_token();
            if let Err(cerr) = token {
                if cerr.get_err_code().eq(&ErrorCode::ErrEOF) {
                    return Ok(ret);
                } else {
                    return Err(cerr);
                }
            }
            ret.push(token?);
        }
    }
    fn next_token(&mut self) -> Result<Token, CompilationError> {
        let mut current = DFAState::InitialState;
        let mut ss = String::new();
        let mut pos = (0, 0);
        loop {
            match current {
                DFAState::InitialState => {
                    let ch = self.next_char().ok_or(CompilationError::new(0, 0, ErrorCode::ErrEOF))?;
                    let mut invalid = false;
                    if ch.is_whitespace() {
                        current = DFAState::InitialState;
                    } else if !ch.is_ascii_graphic() {
                        invalid = true;
                    } else if ch.is_digit(10) {
                        current = DFAState::UnsignedIntegerState;
                    } else if ch.is_ascii_alphabetic() {
                        current = DFAState::IdentifierState;
                    }
                    else {
                        current = match ch {
                            '=' => DFAState::EqualSignState,
                            '-' => DFAState::MinusSignState,
                            '+' => DFAState::PlusSignState,
                            '*' => DFAState::MultiplicationSignState,
                            '/' => DFAState::DivisionSignState,
                            '(' => DFAState::LeftbracketState,
                            ')' => DFAState::RightbracketState,
                            ';' => DFAState::SemicolonState,
                            _ => {
                                invalid = true;
                                DFAState::InitialState
                            }
                        }
                    }
                    if !matches!(current, DFAState::InitialState) {
                        pos = self.previous_pos();
                    }
                    if invalid {
                        self.unread_last();
                        return Result::Err(CompilationError::new_packed(
                            pos,
                            ErrorCode::ErrInvalidInput,
                        ));
                    }
                    if !matches!(current, DFAState::InitialState) {
                        ss.push(ch);
                    }
                }
                DFAState::UnsignedIntegerState => {
                    let current_char = self.next_char();
                    if current_char.map_or(true, |ch| !ch.is_digit(10)) {
                        if current_char.is_some() {
                            self.unread_last();
                        }
                        return Ok(Token::Integer(
                            TokenType::UnsignedInteger,
                            ss.parse().map_err(|_| {
                                CompilationError::new_packed(
                                    self.ptr,
                                    ErrorCode::ErrInvalidIdentifier,
                                )
                            })?,
                            pos,
                            self.ptr,
                        ));
                    } else {
                        ss.push(current_char.unwrap());
                    }
                }
                DFAState::IdentifierState => {
                    let current_char = self.next_char();
                    if current_char.map_or(true, |ch| !ch.is_ascii_alphanumeric()) {
                        if current_char.is_some() {
                            self.unread_last();
                        }
                        return Ok(Token::Str(check_keyword(&ss), ss, pos, self.ptr));
                    } else {
                        ss.push(current_char.unwrap());
                    }
                }

                DFAState::PlusSignState => {
                    return Ok(Token::from_sign(TokenType::PlusSign, pos, self.ptr)?)
                }

                DFAState::MinusSignState => {
                    return Ok(Token::from_sign(TokenType::MinusSign, pos, self.ptr)?)
                }

                DFAState::MultiplicationSignState => {
                    return Ok(Token::from_sign(
                        TokenType::MultiplicationSign,
                        pos,
                        self.ptr,
                    )?)
                }

                DFAState::DivisionSignState => {
                    return Ok(Token::from_sign(TokenType::DivisionSign, pos, self.ptr)?)
                }

                DFAState::EqualSignState => {
                    return Ok(Token::from_sign(TokenType::EqualSign, pos, self.ptr)?)
                }

                DFAState::LeftbracketState => {
                    return Ok(Token::from_sign(TokenType::LeftBracket, pos, self.ptr)?)
                }

                DFAState::RightbracketState => {
                    return Ok(Token::from_sign(TokenType::RightBracket, pos, self.ptr)?)
                }

                DFAState::SemicolonState => {
                    return Ok(Token::from_sign(TokenType::Semicolon, pos, self.ptr)?)
                }
            }
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if self.is_EOF() {
            return None;
        }
        let result = self.lines_buffer[self.ptr.0].as_bytes()[self.ptr.1].into();
        self.ptr = self.next_pos();
        if self.is_EOF() {
            return None;
        }
        Some(result)
    }

    fn previous_pos(&self) -> (usize, usize) {
        if self.ptr == (0, 0) {
            panic!("Unread from beginning!");
        }
        if self.ptr.1 == 0 {
            (self.ptr.0 - 1, self.lines_buffer[self.ptr.0 - 1].len() - 1)
        } else {
            (self.ptr.0, self.ptr.1 - 1)
        }
    }

    fn next_pos(&self) -> (usize, usize) {
        if self.ptr.0 >= self.lines_buffer.len() {
            panic!("Advance after EOF!");
        }
        let mut next = (self.ptr.0, self.ptr.1 + 1);
        while next.0 <= self.lines_buffer.len() - 1 && next.1 >= self.lines_buffer[next.0].as_bytes().len() {
            next = (next.0 + 1, 0);
        }
        next
    }

    fn read_all(&mut self) {
        if self.initialized {
            return;
        }
        let file = File::open(self.filename).expect("cannot find input file");
        for rl in io::BufReader::new(file).lines() {
            self.lines_buffer.push(rl.unwrap());
        }
        self.initialized = true;
    }

    #[allow(non_snake_case)]
    fn is_EOF(&self) -> bool {
        return self.ptr.0 >= self.lines_buffer.len();
    }

    fn unread_last(&mut self) {
        self.ptr = self.previous_pos();
    }
}

fn check_keyword(identifier: &str) -> TokenType {
    match identifier {
        "begin" => TokenType::Begin,
        "end" => TokenType::End,
        "const" => TokenType::Const,
        "var" => TokenType::Var,
        "print" => TokenType::Print,
        _ => TokenType::Identifier,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    NullToken,
    UnsignedInteger,
    Identifier,
    Begin,
    End,
    Var,
    Const,
    Print,
    PlusSign,
    MinusSign,
    MultiplicationSign,
    DivisionSign,
    EqualSign,
    Semicolon,
    LeftBracket,
    RightBracket,
}

impl TokenType {
    pub fn to_string(&self) -> Result<String, CompilationError> {
        Ok(String::from(match self.deref() {
            TokenType::EqualSign => "=",
            TokenType::PlusSign => "+",
            TokenType::MinusSign => "-",
            TokenType::MultiplicationSign => "*",
            TokenType::DivisionSign => "/",
            TokenType::LeftBracket => "(",
            TokenType::RightBracket => ")",
            TokenType::Semicolon => ";",
            _ => return Err(CompilationError::new(0, 0, ErrorCode::ErrInvalidIdentifier)),
        }))
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Integer(TokenType, u32, (usize, usize), (usize, usize)),
    Str(TokenType, String, (usize, usize), (usize, usize)),
}

impl Token {
    pub fn from_sign(
        token_type: TokenType,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Result<Token, CompilationError> {
        let sign_string = token_type.to_string()?;
        Ok(Token::Str(token_type, sign_string, start, end))
    }

    pub fn get_type(&self) -> &TokenType {
        match self {
            Token::Integer(ttype, ..) => ttype,
            Token::Str(ttype, ..) => ttype
        }
    }

    pub fn get_value_string(&self) -> String {
        match self {
            Token::Str(_, string, ..) => String::clone(string),
            Token::Integer(_, int, ..) => int.to_string()
        }
    }

    pub fn get_integer(&self) -> u32 {
        match self {
            Token::Integer(_, int, ..) => *int,
            _ => panic!("You cannot get interger from me !")
        }
    }

    pub fn get_end_pos(&self) -> (usize, usize) {
        match self {
            Token::Str(.., end) => *end,
            Token::Integer(.., end) => *end
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(_, int, ..) => f.write_str(&int.to_string()),
            Token::Str(_, string, ..)  => f.write_str(string)
        }
    }
}

enum DFAState {
    InitialState,
    UnsignedIntegerState,
    PlusSignState,
    MinusSignState,
    DivisionSignState,
    MultiplicationSignState,
    IdentifierState,
    EqualSignState,
    SemicolonState,
    LeftbracketState,
    RightbracketState,
}
