use crate::p1::{PPattern, PTerm};
use crate::{Pattern, Term};
use std::collections::HashMap;

pub(crate) struct Pass2 {
    pspace: Vec<PPattern>,
    vars: Vec<String>,
    k: HashMap<String, usize>,
    k_list: Vec<String>,
}

impl Pass2 {
    pub fn new(pspace: Vec<PPattern>) -> Self {
        Self {
            pspace,
            vars: Vec::new(),
            k: HashMap::from([("Stk".to_string(), 0)]),
            k_list: vec!["Stk".to_string()],
        }
    }

    pub fn parse_t(&mut self, v: PTerm) -> Term {
        match v {
            PTerm::Var(s) => Term::Var(self.get(s)),
            PTerm::K(s) => Term::K(self.get_constant(s)),
            PTerm::Abs(p, n, b) => {
                Self::add(&mut self.vars, &self.pspace[p]);
                let b = self.parse_t(*b);
                self.vars.splice(self.vars.len() - n.., []);
                Term::Abs(p, n, Box::new(b))
            }
            PTerm::App(a, b) => Term::App(
                Box::new(self.parse_t(*a)),
                b.into_iter().map(|i| self.parse_t(i)).collect(),
            ),
            PTerm::Struct(a) => Term::Struct(a.into_iter().map(|i| self.parse_t(i)).collect()),
        }
    }

    pub fn parse_p(&mut self) -> Vec<Pattern> {
        std::mem::take(&mut self.pspace)
            .into_iter()
            .map(|i| self.parse_p_(i))
            .collect()
    }

    pub fn get_k_list(self) -> Vec<String> {
        self.k_list
    }

    fn get(&mut self, s: String) -> usize {
        self.vars.iter().rev().position(|i| i == &s).expect(&s)
    }

    fn get_constant(&mut self, s: String) -> usize {
        *self.k.entry(s.clone()).or_insert_with(|| {
            self.k_list.push(s);
            self.k_list.len() - 1
        })
    }

    fn add(vars: &mut Vec<String>, p: &PPattern) {
        use PPattern::*;

        match p {
            Var(s) => vars.push(s.clone()),
            K(_) => {}
            App(_, a) => a.iter().for_each(|i| Self::add(vars, i)),
        }
    }

    fn parse_p_(&mut self, p: PPattern) -> Pattern {
        use PPattern::*;

        match p {
            Var(_s) => Pattern::Var,
            K(s) => Pattern::K(self.get_constant(s)),
            App(k, a) => Pattern::App(
                self.get_constant(k),
                a.into_iter().map(|i| self.parse_p_(i)).collect(),
            ),
        }
    }
}
