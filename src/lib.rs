pub(crate) mod p1;
pub(crate) mod p2;
pub(crate) mod term;

use p1::parse_term;
use p2::Pass2;
use term::*;

pub struct Vm {
    curr: Term,
    pattern_space: Box<[Pattern]>,
    k_list: Box<[String]>,
}

impl Vm {
    pub fn new(b: Vec<u8>) -> Self {
        let v = &mut b.into_iter();
        let mut pspace = Vec::new();
        let pass1 = parse_term(&mut pspace, v);
        let mut pass2_parser = Pass2::new(pspace);

        let term = pass2_parser.parse_t(pass1);
        let pspace = pass2_parser.parse_p();
        let k = pass2_parser.get_k_list();

        Vm::from(term, pspace, k)
    }

    fn from(term: Term, pspace: Vec<Pattern>, k: Vec<String>) -> Self {
        Self {
            curr: term,
            pattern_space: pspace.into_boxed_slice(),
            k_list: k.into_boxed_slice(),
        }
    }

    pub fn interpret(&mut self) {
        //self.print(&self.curr);
        //println!();
        while self.curr.reduce(&self.pattern_space) {
            //self.print(&self.curr);
            //println!();
        }
        self.print(&self.curr);
        println!();
    }

    fn print(&self, c: &Term) {
        match c {
            Term::Var(x) => print!("{x} "),
            Term::K(k) => print!("{} ", &self.k_list[*k]),
            Term::Struct(a) => {
                for _ in 1..a.len() {
                    print!("|");
                }
                a.iter().for_each(|i| self.print(i));
            }
            Term::App(a, b) => {
                for _ in 0..b.len() {
                    print!("`");
                }
                self.print(a);

                b.iter().for_each(|i| self.print(i));
            }

            Term::Abs(a, _n, b) => {
                print!("\\");
                self.print_p(&self.pattern_space[*a]);
                self.print(b);
            }
        }
    }

    fn print_p(&self, p: &Pattern) {
        match p {
            Pattern::Var => print!("v"),
            Pattern::K(k) => print!("{}", &self.k_list[*k]),
            Pattern::App(k, b) => {
                for _ in 0..b.len() {
                    print!("`");
                }
                print!("{} ", self.k_list[*k]);
                b.iter().for_each(|i| self.print_p(i));
            }
        }
    }
}
