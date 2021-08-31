use std::fs::File;
use std::io::{self, BufRead};
use std::any::Any;

use crate::error::CompilationError;

pub struct Tokenizer<'a> {
    filename : &'a str,
    initialized : bool,
    lines_buffer : Vec<String>,
    ptr : (u64, u64),
    state : DFAState
}

impl<'a> Tokenizer<'a> {
    pub fn NextToken() -> Result<Token, CompilationError> {
        Ok()
    }
    fn readAll(&self) {
        if self.initialized {
            return;
        }
        let file = File::open(self.filename).expect("cannot find target file.");
        self.lines_buffer = io::BufReader::new(file).lines().map(|rl| rl.unwrap()).collect();
    }
}

pub fn tokenize() {

}

enum TokenType {
    NULL_TOKEN,
    UNSIGNED_INTEGER,
    IDENTIFIER,
    BEGIN,
    END,
    VAR,
    CONST,
    PRINT,
    PLUS_SIGN,
    MINUS_SIGN,
    MULTIPLICATION_SIGN,
    DIVISION_SIGN,
    EQUAL_SIGN,
    SEMICOLON,
    LEFT_BRACKET,
    RIGHT_BRACKET
}

struct Token {
    tokenType : TokenType,
    value : Box<dyn Any>,
    start_pos : (u64, u64),
    end_pos : (u64, u64)
}
enum DFAState {
    INITIAL_STATE,
    UNSIGNED_INTEGER_STATE,
    PLUS_SIGN_STATE,
    MINUS_SIGN_STATE,
    DIVISION_SIGN_STATE,
    MULTIPLICATION_SIGN_STATE,
    IDENTIFIER_STATE,
    EQUAL_SIGN_STATE,
    SEMICOLON_STATE,
    LEFTBRACKET_STATE,
    RIGHTBRACKET_STATE
}