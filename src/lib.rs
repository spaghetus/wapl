use std::{collections::HashMap, default, ops::DerefMut, process::Command, str::FromStr};

use strum::EnumString;

pub mod magic;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Command,
    Select,
    Editor,
    String,
    Quote(u32),
    Number,
    Name,
}

#[derive(Debug, Clone, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    #[strum(serialize = ".")]
    Let,
    #[strum(serialize = "+")]
    Add,
    #[strum(serialize = "-")]
    Sub,
    #[strum(serialize = "?")]
    Tern,
    #[strum(serialize = "_")]
    Del,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Datum {
    String(String),
    Quote(String),
    Name(String),
    Number(i32),
    IncompleteKeyword(String),
    Keyword(Keyword),
    #[default]
    Unit,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub stack: Vec<Datum>,
    pub names: HashMap<String, Vec<Datum>>,
    pub mode: Mode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl State {
    pub fn cmd(&mut self, input: char) {
        match (
            self.stack.as_mut_slice(),
            &mut self.mode,
            (self.ctrl, self.alt, self.shift),
            input,
        ) {
            // Command mode
            (_, Mode::Command | Mode::Quote(_), _, c) if c.is_whitespace() => {}
            (_, Mode::Command, _, magic::NAME) => {
                self.stack.push(Datum::Name(String::new()));
                self.mode = Mode::Name;
            }
            (_, Mode::Command, _, magic::NUMBER) => {
                self.stack.push(Datum::Number(0));
                self.mode = Mode::Number;
            }
            (_, Mode::Command, _, magic::QUOTE) => {
                self.stack.push(Datum::Quote(String::new()));
                self.mode = Mode::Quote(0);
            }
            ([.., Datum::Quote(q)], Mode::Command, _, magic::FOLLOW) => {
                let q = std::mem::take(q);
                self.stack.pop();
                q.chars().for_each(|c| self.cmd(c));
            }
            ([.., Datum::Name(n)], Mode::Command, _, magic::FOLLOW)
                if self.names.contains_key(n) =>
            {
                let datum = self.names.get(n).unwrap();
                self.stack.pop();
                self.stack.push(datum.iter().last().cloned().unwrap());
            }
            ([.., Datum::IncompleteKeyword(k)], Mode::Command, _, c) => {
                k.push(c);
                if let Ok(k) = Keyword::from_str(k) {
                    self.stack.pop();
                    self.stack.push(Datum::Keyword(k));
                }
            }
            (_, Mode::Command, _, c) => {
                self.stack.push(Datum::IncompleteKeyword(String::new()));
                self.cmd(c)
            }
            // Name mode
            (_, Mode::Name, _, magic::ESC) => self.mode = Mode::Command,
            ([.., Datum::Name(n)], Mode::Name, _, c) => n.push(c),
            // Number mode
            (_, Mode::Number, _, magic::ESC) => self.mode = Mode::Command,
            ([.., Datum::Number(n)], Mode::Number, _, c) => {
                if let Some(d) = c.to_digit(10) {
                    *n *= 10;
                    *n += d as i32;
                }
                if c == '-' {
                    *n *= -1;
                }
            }
            // Quote mode
            ([.., Datum::Quote(_)], Mode::Quote(0), _, magic::ESC) => {
                self.mode = Mode::Command;
            }
            ([.., Datum::Quote(quoted)], Mode::Quote(level), _, c) => {
                match c {
                    magic::NUMBER | magic::NAME | magic::STRING | magic::QUOTE => *level += 1,
                    magic::ESC => *level -= 1,
                    _ => {}
                }
                quoted.push(c)
            }
            c => unimplemented!("{c:#?}"),
        }
        if !input.is_whitespace() {
            // eprintln!("state: {input} -> {self:#?}");
            if self.mode == Mode::Command {
                while self.eval() {}
            }
        }
    }
    pub fn eval(&mut self) -> bool {
        eprintln!("evaluating: {:#?}", self.stack);
        match self.stack.as_mut_slice() {
            // Arithmetic
            [.., Datum::Number(lhs), Datum::Number(rhs), Datum::Keyword(Keyword::Add)] => {
                let lhs = *lhs;
                let rhs = *rhs;
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                self.stack.push(Datum::Number(lhs + rhs))
            }
            [.., Datum::Number(lhs), Datum::Number(rhs), Datum::Keyword(Keyword::Sub)] => {
                let lhs = *lhs;
                let rhs = *rhs;
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                self.stack.push(Datum::Number(lhs - rhs))
            }
            // Let expressions
            [.., value, Datum::Name(name), Datum::Keyword(Keyword::Let)] => {
                let name = std::mem::take(name);
                let value = std::mem::take(value);
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                self.names.entry(name).or_default().push(value);
            }
            [.., Datum::Name(name), Datum::Keyword(Keyword::Del)] => {
                let name = std::mem::take(name);
                self.stack.pop();
                self.stack.pop();
                self.names.entry(name).or_default().pop();
            }
            // Ternary expressions
            [.., falsy, truthy, Datum::Number(condition), Datum::Keyword(Keyword::Tern)] => {
                let falsy = falsy.clone();
                let truthy = truthy.clone();
                let condition = *condition;
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                self.stack.push(if condition > 0 { truthy } else { falsy })
            }
            _ => return false,
        }
        true
    }
}

#[test]
fn let_st() {
    let program = "
        #123⎋ $n⎋.
        $n⎋~
    ";
    let mut machine = State::default();
    program.chars().for_each(|c| {
        machine.cmd(c);
    });
    assert_eq!(
        machine.stack.last(),
        Some(&Datum::Number(123)),
        "Test failed with state {machine:#?}"
    );
}

#[test]
fn functions() {
    let program = "
            `$rhs⎋.
            $lhs⎋.
            $lhs⎋~$rhs⎋~+⎋
            $add⎋.
        #1⎋ #2⎋ $add⎋~~
    ";
    let mut machine = State::default();
    program.chars().for_each(|c| {
        machine.cmd(c);
    });
    assert_eq!(
        machine.stack.last(),
        Some(&Datum::Number(3)),
        "Test failed with state {machine:#?}"
    );
}

#[test]
fn fib() {
    fn reference(n: i32) -> i32 {
        match n {
            ..=0 => 1,
            n => reference(n - 1) + reference(n - 2),
        }
    }
    let program = "
            `
                $d⎋.
                `#1⎋⎋
                `
                    $d⎋~#1⎋-$f⎋~~
                    $d⎋~#2⎋-$f⎋~~
                    +
                ⎋
                $d⎋~
                ?~
                $d⎋_
            ⎋
            $f⎋.
        #10⎋ $f⎋~~
    ";
    let mut machine = State::default();
    program.chars().for_each(|c| {
        machine.cmd(c);
    });
    assert_eq!(
        machine.stack.last(),
        Some(&Datum::Number(reference(10))),
        "Test failed with state {machine:#?}"
    );
}
