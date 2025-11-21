use shadowjs_engine::ShadowEngine;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: shadowjs <file.js>");
        return;
    }

    let src = fs::read_to_string(&args[1]).unwrap();
    let mut engine = ShadowEngine::new();

    if let Err(e) = engine.eval(&src) {
        eprintln!("Error: {}", e);
    }
}
