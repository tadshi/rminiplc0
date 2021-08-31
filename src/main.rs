extern crate args;
extern crate getopts;

use args::{Args, ArgsError};

const PROGRAM_DESC: &'static str = "A Rust version for miniplc0 complier!";
const PROGRAM_NAME: &'static str = "rMINIPLC0c";

use std::{env};

mod analyzer;
mod tokenizer;
mod error;
use analyzer::analyze;
use tokenizer::tokenize;

enum Modules {
    TOKENIZE, ANALYZE, NOTHING
}

struct Target {
    task : Modules,
    path : String
}

fn main() {
    println!("Hello, world!");
    let target = parse(&env::args().collect()).expect("Please check your command line.");
    match target.task {
        Modules::ANALYZE => analyze(),
        Modules::TOKENIZE => tokenize(),
        Modules::NOTHING => ()
    };
}

fn parse(input :&Vec<String>) -> Result<Target, ArgsError> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.flag("h", "help", "Print this");
    args.option("i", "input string", "The input file. The default is os.Stdin. (default \"-\")", "NAME", getopts::Occur::Req, Some(String::from("os.Stdin")));
    args.parse(input)?;
    let help = args.value_of("help")?;
    if help {
        args.full_usage();
        return Ok(Target{task: Modules::NOTHING, path: String::from("773")});
    }
    Ok(Target{task: Modules::TOKENIZE, path: String::from("what")})
}