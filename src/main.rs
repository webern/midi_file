use rust_template::add;

fn main() -> ! {
    if let Err(e) = add(1, 2) {
        eprintln!("{}", e);
        std::process::exit(1)
    } else {
        std::process::exit(0)
    }
}
