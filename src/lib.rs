mod analyzer;
mod tokenizer;
mod error;

pub use analyzer::analyze;
pub use tokenizer::tokenize;

#[cfg(test)]
mod tests{
    use crate::tokenizer::tokenize;

    #[test]
    fn test_tokenizer() {
        let tokens =  tokenize(String::from("files/somhow.plc0"));
        for token in tokens {
            print!("{}", &token);
        }
    }
}