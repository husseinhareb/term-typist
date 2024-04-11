use std::env;
extern crate rustbox;

use rustbox::{Color, Key, RustBox};
use std::io::{self, Write};
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
        let initial_text = generator::generate_random_sentence(30).to_string();
        println!("{}",initial_text);
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