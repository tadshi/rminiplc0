use std::{fmt};

pub fn analyze(input: String) -> Vec<Instruction> {
    Vec::new()
}

pub struct Instruction {
    opr: Operation,
    x: u32
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match f {
            _ => f.write_str("todo!")
        }
    }
}

#[derive(Debug)]
pub enum Operation{
    ILL,
    LIT,
    LOD,
    STO,
    ADD,
    SUB,
    MUL,
    DIV,
    WRT
}