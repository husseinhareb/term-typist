// src/main.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Delegate entirely to the library crate
    term_typist::run()
}
