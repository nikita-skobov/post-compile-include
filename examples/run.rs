use post_compile_include::get_all_data_sections;

fn main() {
    let data = include_bytes!("./include.txt");
    let data_map = get_all_data_sections(data).expect("Failed to get data sections");
    println!("{:#?}", data_map);
}
