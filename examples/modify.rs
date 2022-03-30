use std::io::{Read, Write};

use post_compile_include::{write_to_included_section, DataToWrite};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let first = args.first().expect("Must provide path to a file");
    let mut file_data = vec![];
    let mut f = std::fs::File::open(&first).expect("Failed to open file");
    f.read_to_end(&mut file_data).expect("Failed to read to vec");
    write_to_included_section(&mut file_data, vec![
        DataToWrite { key: "hi".to_string(), data: vec![1, 2, 3] }
    ]).expect("Failed to write included section");

    drop(f);
    let mut outf = std::fs::File::create(&first).expect("Failed to open file");
    outf.write_all(&file_data[..]).expect("Failed to write out file");
}
