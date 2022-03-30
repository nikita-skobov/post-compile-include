use post_compile_include::generate_included_data_file;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let first = args.first().expect("Must provide path to a file");
    if let Err(e) = generate_included_data_file(first, 2) {
        eprintln!("{e}");
    }
}
