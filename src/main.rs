use std::env;

mod config;
mod generator;
mod ui;
mod wpm;

fn help() {
    println!("Usage: term-typist [options] | term-typist");
    println!("Options:");   
    println!("-h               Display this help message");
    println!("-w <number>      Set the number of words");     
  
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
            "-w" => {
                if let Some(nb_of_words) = iter.next() {
                    match nb_of_words.parse::<i32>() {
                        Ok(nb) => {
                            let _ = config::write_nb_of_words(nb);
                        }
                        Err(_) => {
                            eprintln!("Invalid value provided for -s flag: {}", nb_of_words);
                            help();
                            return;
                        }
                    }
                } else {
                    eprintln!("Unit value not provided for the -s flag.");
                    help();
                    return;
                }
            }
            _ => {
                eprintln!("Invalid argument: {}", arg);
                help();
                return;
            }
        }
    }
}