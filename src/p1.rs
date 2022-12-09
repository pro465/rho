#[derive(Clone)]
pub(crate) enum PTerm {
    Var(String),
    Abs(String, Box<PTerm>),
    App(Box<PTerm>, Vec<PTerm>),
}

pub(crate) fn parse_term(iter: &mut (impl Iterator<Item = u8> + Clone)) -> PTerm {
    match iter.next().unwrap() {
        b'`' => match parse_term(iter) {
            PTerm::App(f, mut a) => {
                a.push(parse_term(iter));
                PTerm::App(f, a)
            }
            x => PTerm::App(Box::new(x), vec![parse_term(iter)]),
        },
        b'\\' => {
            if let PTerm::Var(ident) = parse_term(iter) {
                PTerm::Abs(ident, Box::new(parse_term(iter)))
            } else {
                println!("invalid syntax: expected identifier after abstraction symbol");
                std::process::exit(-1)
            }
        }
        x if x.is_ascii_alphabetic() => PTerm::Var(parse_ident(x, iter)),

        _ => parse_term(iter),
    }
}

fn parse_ident(first: u8, iter: &mut (impl Iterator<Item = u8> + Clone)) -> String {
    let mut s: String = (first as char).try_into().unwrap();
    let mut iter_clone = iter.clone();
    while let Some(x) = iter_clone.next() {
        if !x.is_ascii_alphanumeric() {
            break;
        }

        s.push(x.try_into().unwrap());
        iter.next().unwrap();
    }
    s
}
