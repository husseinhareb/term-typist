use std::env;

mod config;
mod generator;
mod ui;

fn help() {
    println!("Usage: term-typist [options] | term-typist");
    println!("Options:");   
    println!("-h               Display this help message");     
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        let _ = config::create_config();
        let _ = ui::listen_for_alphabets();
        return;
    }

    let mut iter = args.iter().skip(1); // Skip the first argument (program name)

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" => {
                help();
                return;
            }
            _ => {
                eprintln!("Invalid argument: {}", arg);
                help();
                return;
            }
        }
    }
}