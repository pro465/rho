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
        match self {
            Term::Abs(.., b) => b.reduce(pattern_space),
            Term::Struct(a) => {
                let len = a.len();
                let changed = a.iter_mut().fold(false, |a, i| i.reduce(pattern_space) | a);
                a.retain(|i| !i.is_stk());
                if a.is_empty() {
                    self.make_stk()
                } else {
                    (len != a.len()) | changed
                }
            }
            Term::App(f, a) => {
                if !f.reduce(pattern_space) {
                    if let Term::App(_, a_) = &mut **f {
                        a_.append(a);
                        *self = f.take();
                        return true;
                    }

                    if f.is_stk() {
                        return self.make_stk();
                    }

                    let res = a.iter_mut().fold(false, |r, i| i.reduce(pattern_space) | r);

                    if is_val_rho_delta(&a[0]) {
                        return match &mut **f {
                            Term::Abs(p, n, b) => {
                                apply(&pattern_space[*p], *n, b, a.remove(0));
                                **f = b.take();

                                if a.is_empty() {
                                    *self = f.take();
                                }

                                true
                            }
                            Term::Struct(s) => {
                                s.iter_mut()
                                    .for_each(|i| *i = Term::App(Box::new(i.take()), a.clone()));
                                *self = f.take();
                                true
                            }
                            _ => res,
                        };
                    }

                    let mut a_0 = a[0].take();

                    if is_val_gamma(&a_0) {
                        if let Term::Struct(s) = &mut a_0 {
                            a.remove(0);
                            s.iter_mut().for_each(|i| {
                                *i = Term::App(
                                    f.clone(),
                                    std::iter::once(i.take())
                                        .chain(a.iter().map(Clone::clone))
                                        .collect(),
                                )
                            });
                            *f = Box::new(a_0);
                            if a.is_empty() {
                                *self = f.take();
                            }
                            true
                        } else {
                            a[0] = a_0;
                            res
                        }
                    } else {
                        a[0] = a_0;
                        res
                    }
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    fn take(&mut self) -> Term {
        use std::mem::replace;
        replace(self, Term::stk())
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

fn is_val<const C: bool>(t: &Term) -> bool {
    match t {
        Term::Var(_) => true,
        Term::App(a, b) => {
            (match &**a {
                Term::K(_) => true,
                Term::App(..) => is_val::<C>(a),
                _ => false,
            }) && b.iter().all(is_val::<C>)
        }
        Term::Abs(..) => true,
        Term::K(0) => C,
        Term::K(_) => true,
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

fn apply(p: &Pattern, n: usize, b: &mut Term, a: Term) {
    let mut matches = Vec::with_capacity(n);

    if try_match(p, a, &mut matches).is_err() {
        b.make_stk();
        return;
    }

    if matches.is_empty() {
        return;
    }

    substitute_matches(0, b, &matches);

    sub(matches.len(), matches.len(), b);
}

fn try_match(p: &Pattern, e: Term, matches: &mut Vec<Term>) -> Result<(), ()> {
    match p {
        Pattern::App(p_1, a_1) => match e {
            Term::App(p_2, a_2) if a_1.len() == a_2.len() => match &*p_2 {
                Term::K(p_2) if p_2 == p_1 => a_1
                    .iter()
                    .zip(a_2.into_iter())
                    .fold(Ok(()), |r, (a_1, a_2)| {
                        r.and_then(|()| try_match(a_1, a_2, matches))
                    }),
                _ => Err(()),
            },

            _ => Err(()),
        },
        Pattern::K(k) => match e {
            Term::K(x) if x == *k => Ok(()),
            _ => Err(()),
        },
        Pattern::Var => {
            matches.push(e);
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
