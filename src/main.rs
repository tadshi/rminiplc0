extern crate args;
extern crate getopts;

use args::{Args, ArgsError};

const PROGRAM_DESC: &'static str = "A Rust version for miniplc0 complier!";
const PROGRAM_NAME: &'static str = "rMINIPLC0c";

use std::{env, fs::File, io::{BufWriter, Write}};

mod analyzer;
mod error;
mod tokenizer;
use analyzer::analyze;
use tokenizer::tokenize;

enum Modules {
    TOKENIZE,
    ANALYZE,
    NOTHING,
}

struct Target {
    task: Modules,
    input: String,
    output: String,
}

fn main() {
    println!("Hello, world!");
    let target = parse(&env::args().collect()).expect("Please check your command line.");
    if matches!(target.task, Modules::NOTHING) {
        return;
    }
    let mut writer = BufWriter::new(File::create(target.output).expect("unable to open output file"));
    match target.task {
        Modules::ANALYZE => analyze(target.input).iter().for_each(|instr| {write!(writer, "{}\n", instr).unwrap();return;}),
        Modules::TOKENIZE => tokenize(target.input).iter().for_each(|token| {write!(writer, "{}\n", token).unwrap();return;}),
        Modules::NOTHING => return
    }
}

fn parse(input: &Vec<String>) -> Result<Target, ArgsError> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.flag("h", "help", "Print this");
    args.option(
        "i",
        "input",
        "The input file. The default is os.Stdin. (default \"-\")",
        "NAME",
        getopts::Occur::Req,
        Some(String::from("os.Stdin")),
    );
    args.option(
        "o",
        "output",
        "output file",
        "set output file",
        getopts::Occur::Req,
        Some(String::from("a.out")),
    );
    args.flag("t", "tokenize", "perform tokenization");
    args.flag("l", "analyze", "perform analyzation");
    args.parse(input)?;
    let help = args.value_of("help")?;
    let input: String = args.value_of("input")?;
    let output: String = args.value_of("output")?;
    if help {
        print!("{}", args.full_usage());
        return Ok(Target {
            task: Modules::NOTHING,
            input: String::new(),
            output: String::new(),
        });
    }

    if args.value_of("tokenize")? {
        return Ok(Target {
            task : Modules::TOKENIZE,
            input,
            output
        })
    }

    if args.value_of("analyze")? {
        return Ok(Target {
            task: Modules::ANALYZE,
            input,
            output
        });
    }

    print!("{}", args.full_usage());
    return Ok(Target {
        task: Modules::NOTHING,
        input: String::new(),
        output: String::new(),
    });
}
