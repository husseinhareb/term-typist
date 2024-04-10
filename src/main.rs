mod generator;

fn main() {
    let sentence = generator::generate_random_sentence(3000);
    println!("{}", sentence);
}
