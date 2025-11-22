use shadowjs_engine::ShadowEngine;
use std::env;
use std::fs;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: shadowjs <file.js> [--bench] [--debug]");
        return;
    }

    let mut filename = String::new();
    let mut bench = false;
    let mut debug = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "--bench" => bench = true,
            "--debug" => debug = true,
            _ => {
                if filename.is_empty() {
                    filename = arg.clone();
                }
            }
        }
    }

    if filename.is_empty() {
        eprintln!("Usage: shadowjs <file.js> [--bench] [--debug]");
        return;
    }

    let src = fs::read_to_string(&filename).unwrap();
    let mut engine = ShadowEngine::new();
    engine.set_debug(debug);

    let start = Instant::now();
    if let Err(e) = engine.eval(&src) {
        eprintln!("Error: {}", e);
    }
    let duration = start.elapsed();

    if bench {
        let time_str = if duration.as_secs() > 0 {
            format!("{:.3}s", duration.as_secs_f64())
        } else if duration.as_millis() > 0 {
            format!("{}ms", duration.as_millis())
        } else if duration.as_micros() > 0 {
            format!("{}µs", duration.as_micros())
        } else {
            format!("{}ns", duration.as_nanos())
        };
        println!("Execution time: {}", time_str);
    }
}
