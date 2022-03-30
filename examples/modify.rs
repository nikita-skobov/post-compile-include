use post_compile_include::{DataToWrite, write_to_compiled_file};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let first = args.first().expect("Must provide path to a file");
    let write_data = vec![
        DataToWrite { key: "hi".to_string(), data: vec![1, 2, 3] }
    ];
    if let Err(e) = write_to_compiled_file(first, write_data) {
        eprintln!("{}", e);
    }
}
