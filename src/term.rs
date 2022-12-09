use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Instr {
    App(usize),
    Var(usize),
    Abs(Rc<Block>),
}

#[derive(Clone, Debug, Default)]
pub struct Block(pub(crate) Vec<Instr>);

use Instr::*;

impl Block {
    pub fn reduce(self) -> Block {
        let mut ret = Vec::new();

        for c in self.0.into_iter() {
            match c {
                App(n) => {
                    let mut f = take(ret.pop().unwrap());
                    for _ in 0..n {
                        let a = ret.pop().unwrap();

                        f.substitute(0, a);
                        f = f.reduce();
                    }
                    ret.push(Rc::new(f))
                }
                Abs(f) => ret.push(f),
                _ => unreachable!(),
            }
        }

        take(ret.pop().unwrap())
    }

    fn substitute(&mut self, dom: usize, a: Rc<Block>) {
        for b in &mut self.0 {
            match b {
                Var(x) if dom == *x => *b = Instr::Abs(a.clone()),
                Abs(b) => Rc::make_mut(b).substitute(dom + 1, a.clone()),
                _ => {}
            }
        }
    }

    pub fn print(&self) {
        use Instr::*;

        print!("(");
        print!("\\");
        self.0.iter().rev().for_each(|c| match c {
            Var(x) => print!("{x} "),
            App(x) => {
                for _ in 0..*x {
                    print!("`")
                }
            }
            Abs(b) => {
                b.print();
            }
        });
        print!(")");
    }
}

fn take(mut b: Rc<Block>) -> Block {
    std::mem::take(Rc::make_mut(&mut b))
}
