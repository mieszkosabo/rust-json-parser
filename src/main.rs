use std::{
    fs::{self, File},
    io::Read,
};

fn main() {
    let contents =
        fs::read_to_string("benchmarks/input.json").expect("Something went wrong reading the file");

    print!("1");
}
