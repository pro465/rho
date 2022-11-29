#[derive(Clone, Debug)]
pub(crate) enum Pattern {
    Var,
    K(usize),
    App(usize, Vec<Pattern>),
}

#[derive(Clone, Debug)]
pub(crate) enum Term {
    Var(usize),
    K(usize),
    Abs(usize, usize, Box<Term>),
    App(Box<Term>, Vec<Term>),
    Struct(Vec<Term>),
}

impl Term {
    pub(crate) fn reduce(&mut self, pattern_space: &[Pattern]) -> bool {
        use std::mem::{replace, take};

        match self {
            Term::App(f, a) => match &mut **f {
                Term::Abs(p, _l, b) => {
                    while a[0].reduce(pattern_space) {}

                    let mut v = Vec::new();
                    apply(&mut v, &pattern_space[*p], &b, &a[0]);

                    a.remove(0);

                    let f = if v.len() > 1 {
                        Term::Struct(v)
                    } else {
                        v.pop().expect("vector should not be empty")
                    };

                    *self = if !a.is_empty() {
                        Term::App(Box::new(f), take(a))
                    } else {
                        f
                    };

                    true
                }
                Term::K(0) => self.make_stk(),
                Term::App(f_, a_) => {
                    a_.append(a);
                    *self = Term::App(replace(f_, Box::new(Term::Struct(Vec::new()))), take(a_));
                    true
                }
                Term::Struct(s) => {
                    let s = take(s);
                    *self = Term::Struct(
                        s.into_iter()
                            .map(|i| Term::App(Box::new(i), a.clone()))
                            .collect(),
                    );
                    true
                }
                _ => false,
            },

            Term::Struct(x) => {
                let res = x.iter_mut().fold(false, |a, i| i.reduce(pattern_space) | a);
                let len_before = x.len();
                x.retain(|i| !i.is_stk());
                let len_after = x.len();
                if len_before > len_after {
                    if len_after == 0 {
                        self.make_stk();
                    }

                    true
                } else {
                    res
                }
            }

            Term::Abs(_p, _n, b) => b.reduce(pattern_space),

            _ => false,
        }
    }

    fn make_stk(&mut self) -> bool {
        *self = Term::stk();
        true
    }

    fn stk() -> Self {
        Term::K(0)
    }

    fn is_stk(&self) -> bool {
        matches!(self, Term::K(0))
    }
}

fn is_val<const C1: bool>(t: &Term) -> bool {
    match t {
        Term::Var(_) => true,
        Term::K(x) if C || !t.is_stk() => true,
        Term::App(a, b) => match a {
            Term::K(_) => true,
            Term::App(..) => is_val(a),
            _ => false,
        } && b.iter().all(is_val),
        Term::Abs(..) => true,
        _ => false,
    }
}

fn is_val_rho_delta(t: &Term) -> bool {
    is_val::<true>(t)
}

fn is_val_gamma(t: &Term) -> bool {
    match t {
        Term::Struct(a) => a.iter().all(is_val_gamma),
        _ => is_val::<false>(t),
    }
}

fn apply(v: &mut Vec<Term>, p: &Pattern, b: &Term, a: &Term) {
    match a {
        Term::Struct(a) => a.iter().for_each(|i| apply(v, p, b, i)),
        _ => v.push(apply_val(p, b, a)),
    }
}

fn apply_val(p: &Pattern, b: &Term, a: &Term) -> Term {
    let mut matches = Vec::new();

    if try_match(p, a, &mut matches).is_err() {
        return Term::stk();
    }

    if matches.is_empty() {
        return b.to_owned();
    }

    let mut b = b.clone();

    substitute_matches(0, &mut b, &matches);

    sub(matches.len(), matches.len(), &mut b);

    b
}

fn try_match(p: &Pattern, e: &Term, matches: &mut Vec<Term>) -> Result<(), ()> {
    match p {
        Pattern::App(p_1, a_1) => match e {
            Term::App(p_2, a_2) if a_2.len() == a_2.len() => match &**p_2 {
                Term::K(p_2) if p_2 == p_1 => {
                    a_1.iter().zip(a_2.iter()).fold(Ok(()), |r, (a_1, a_2)| {
                        r.and_then(|()| try_match(a_1, a_2, matches))
                    })
                }
                _ => Err(()),
            },

            _ => Err(()),
        },
        Pattern::K(k) => match e {
            Term::K(x) if x == k => Ok(()),
            _ => Err(()),
        },
        Pattern::Var => {
            matches.push(e.clone());
            Ok(())
        }
    }
}

fn substitute_matches(dom: usize, b: &mut Term, matches: &[Term]) {
    match b {
        Term::Var(x) if (dom..dom + matches.len()).contains(x) => {
            *b = add(
                0,
                dom + matches.len(),
                matches[matches.len() - 1 - (*x - dom)].to_owned(),
            )
        }
        Term::Struct(a) => a
            .iter_mut()
            .for_each(|i| substitute_matches(dom, i, matches)),
        Term::App(a, b) => {
            substitute_matches(dom, a, matches);
            b.iter_mut()
                .for_each(|i| substitute_matches(dom, i, matches))
        }
        Term::Abs(_p, num, b) => substitute_matches(dom + *num, b, matches),
        _ => {}
    }
}

fn add(thres: usize, incr: usize, t: Term) -> Term {
    use Term::*;

    match t {
        Var(x) if x >= thres => Var(x + incr),
        Struct(x) => Struct(x.into_iter().map(|i| add(thres, incr, i)).collect()),
        App(a, x) => App(
            Box::new(add(thres, incr, *a)),
            x.into_iter().map(|i| add(thres, incr, i)).collect(),
        ),
        Abs(p, n, b) => Abs(p, n, Box::new(add(thres + n, incr, *b))),
        k => k,
    }
}

fn sub(thres: usize, decr: usize, t: &mut Term) {
    use Term::*;

    match t {
        Var(x) if *x >= thres => *x -= decr,
        Struct(x) => x.iter_mut().for_each(|i| sub(thres, decr, i)),
        App(a, x) => {
            sub(thres, decr, a);
            x.iter_mut().for_each(|i| sub(thres, decr, i));
        }
        Abs(_p, n, b) => sub(thres + *n, decr, b),
        _ => {}
    }
}
