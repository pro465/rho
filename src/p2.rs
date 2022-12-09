use crate::p1::PTerm;
use crate::{Block, Instr};

pub(crate) struct Pass2 {
    vars: Vec<String>,
}

impl Pass2 {
    pub fn new() -> Self {
        Self { vars: Vec::new() }
    }

    pub fn parse(&mut self, v: PTerm, b: &mut Vec<Instr>) {
        use std::rc::Rc;

        let instr = match v {
            PTerm::Var(s) => Instr::Var(self.get(s)),
            PTerm::Abs(p, b) => {
                self.vars.push(p);
                let mut res = Vec::new();
                self.parse(*b, &mut res);
                self.vars.pop().unwrap();

                Instr::Abs(Rc::new(Block(res)))
            }
            PTerm::App(f, a) => {
                let num_app = a.len();

                for i in a.into_iter().rev() {
                    self.parse(i, b);
                }
                self.parse(*f, b);
                Instr::App(num_app)
            }
        };

        b.push(instr);
    }

    fn get(&mut self, s: String) -> usize {
        self.vars.iter().rev().position(|i| i == &s).expect(&s)
    }
}
