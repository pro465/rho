use rho::Vm;
use std::fs;

fn main() {
    Vm::new(
        fs::read(
            fs::canonicalize(std::env::args().nth(1).unwrap_or_else(|| help()))
                .expect("could not canonicalize argument"),
        )
        .expect("could not read file"),
    )
    .interpret()
}

fn help() -> ! {
    println!(
        "usage: {} <filename>",
        std::env::current_exe().unwrap_or("ctfuck".into()).display()
    );
    std::process::exit(-1);
}
