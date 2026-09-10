use std::collections::HashMap;
use termview::{
    parsing::{Parser, Tokenizer, TokenNode},
};

fn main() -> Result<(), String> {
    let source = "2 * x + y";
    println!("{}", source);

    let mut tokenizer = Tokenizer::new(source);

    let tokens = tokenizer.tokenize().unwrap();
    
    for token in &tokens {
        println!("{:?}", token);
    }

    let mut parser = Parser::new(tokens);

    let tree: TokenNode = parser.parse_expression(0)?;

    println!("\n{:?}", tree);

    let vars = HashMap::from([
        ("x".to_string(), 1.0),
        ("y".to_string(), 2.0),
    ]);

    let result = tree.evaluate(&vars);

    println!("\nResult: {}", result);

    Ok(())
}