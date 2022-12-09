#[derive(Clone)]
pub(crate) enum PPattern {
    Var(String),
    K(String),
    App(String, Vec<PPattern>),
}

#[derive(Clone)]
pub(crate) enum PTerm {
    Var(String),
    K(String),
    Abs(usize, usize, Box<PTerm>),
    App(Box<PTerm>, Vec<PTerm>),
    Struct(Vec<PTerm>),
}

pub(crate) fn parse_term(
    pspace: &mut Vec<PPattern>,
    iter: &mut (impl Iterator<Item = u8> + Clone),
) -> PTerm {
    match iter.next().unwrap() {
        b'`' => match parse_term(pspace, iter) {
            PTerm::App(f, mut a) => {
                a.insert(0, parse_term(pspace, iter));
                PTerm::App(f, a)
            }
            x => PTerm::App(Box::new(x), vec![parse_term(pspace, iter)]),
        },
        b'\\' => {
            let idx = pspace.len();
            let (num, pattern) = parse_pattern(iter);
            pspace.push(pattern);
            PTerm::Abs(idx, num, Box::new(parse_term(pspace, iter)))
        }
        b'|' => match parse_term(pspace, iter) {
            PTerm::Struct(mut s) => {
                s.push(parse_term(pspace, iter));
                PTerm::Struct(s)
            }
            x => PTerm::Struct(vec![x, parse_term(pspace, iter)]),
        },
        x if x.is_ascii_uppercase() => PTerm::K(parse_ident(x, iter)),
        x if x.is_ascii_lowercase() => PTerm::Var(parse_ident(x, iter)),

        _ => parse_term(pspace, iter),
    }
}

fn parse_pattern(iter: &mut (impl Iterator<Item = u8> + Clone)) -> (usize, PPattern) {
    match iter.next().unwrap() {
        b'`' => match parse_pattern(iter) {
            (n1, PPattern::App(k, mut b)) => {
                let (n2, p) = parse_pattern(iter);
                b.insert(0, p);
                (n1 + n2, PPattern::App(k, b))
            }

            (0, PPattern::K(k)) => {
                let (n, p) = parse_pattern(iter);

                (n, PPattern::App(k, vec![p]))
            }

            _ => panic!("invalid syntax: patterns only support applications to constants"),
        },
        x if x.is_ascii_uppercase() => (0, PPattern::K(parse_ident(x, iter))),
        x if x.is_ascii_lowercase() => (1, PPattern::Var(parse_ident(x, iter))),

        _ => parse_pattern(iter),
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
