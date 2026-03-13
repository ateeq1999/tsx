use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    // The first argument is the path to the executable
    if args.len() > 1 {
        let user_input = &args[1];
        println!("You provided the argument: {}", user_input);
    } else {
        println!("No argument provided. Try running with `cargo run -- <some_input>`");
    }
}
