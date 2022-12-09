use std::fs;

fn main() {
    lam::parse(
        fs::read(
            fs::canonicalize(std::env::args().nth(1).unwrap_or_else(|| help()))
                .expect("could not canonicalize argument"),
        )
        .expect("could not read file"),
    )
    .reduce()
    .print();
    println!()
}

fn help() -> ! {
    println!(
        "usage: {} <filename>",
        std::env::current_exe().unwrap_or("lam".into()).display()
    );
    std::process::exit(-1);
}
