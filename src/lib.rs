use core::fmt;
use std::{
    collections::{vec_deque, VecDeque},
    fmt::{write, Display, Formatter},
};

use pp::BufPrinter;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sexp {
    Atom(String),
    List(Vec<Sexp>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum FormatToken<'a> {
    Open(),
    Close(),
    Breakhint(),
    Str(&'a String),
}

fn pp_tokenize(sexp: &Sexp) -> Vec<FormatToken> {
    let mut tokens: Vec<FormatToken> = vec![FormatToken::Open()];

    enum StackToken<'a> {
        Open(),
        Breakhint(),
        Close(),
        Sexp(&'a Sexp),
    }

    let mut stack: Vec<StackToken> = vec![StackToken::Sexp(sexp)];

    while let Some(s) = stack.pop() {
        match s {
            StackToken::Sexp(s) => match s {
                Sexp::Atom(w) => tokens.push(FormatToken::Str(w)),
                Sexp::List(l) => {
                    stack.push(StackToken::Close());
                    let (head, tail) = l.split_first().unwrap();
                    stack.extend(tail.iter().rev().flat_map(|x| {
                        vec![StackToken::Sexp(x), StackToken::Breakhint()].into_iter()
                    }));
                    stack.push(StackToken::Sexp(head));
                    stack.push(StackToken::Open());
                }
            },
            StackToken::Open() => tokens.push(FormatToken::Open()),
            StackToken::Close() => tokens.push(FormatToken::Close()),
            StackToken::Breakhint() => tokens.push(FormatToken::Breakhint()),
        }
    }
    tokens.push(FormatToken::Close());
    tokens
}

fn pp_hum_indent(ppf: &mut BufPrinter, sexp: &Sexp, indent: usize) {
    let tokens = pp_tokenize(sexp);
    tokens.into_iter().for_each(|t| match t {
        FormatToken::Open() => {
            ppf.open_box(indent);
            ppf.print_string("(");
        }
        FormatToken::Close() => {
            ppf.print_string(")");
            ppf.close_box();
        }
        FormatToken::Breakhint() => ppf.print_space(),
        FormatToken::Str(s) => ppf.print_string(s),
    });

    ppf.print_flush();
}

impl Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = 2;
        let mut ppf = pp::BufPrinter::new(78, 10, 68, 100000);
        let pp_f = |ppf: &mut BufPrinter, sexp: &Sexp| pp_hum_indent(ppf, sexp, indent);
        ppf.fprintf(pp_f, self);
        write!(f, "{}", ppf.out_buf)
    }
}

macro_rules! Sexp {
    () => {};
}

macro_rules! List {
    ( $($e:expr),* ) => {
        {
            let mut v = Vec::new();
            $( v.push($e); )*
            Sexp::List(v)
        }
    };
}

macro_rules! Atom {
    ($e:expr) => {
        Sexp::Atom($e.to_string())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let simple_sexp = List!(Atom!("symbol_id"), Atom!("1"));

        println!("{}", simple_sexp)
        // Expected:
        // ((symbol_id 1))
    }

    #[test]
    fn test_sexp_longer() {
        let longer_sexp = List!(
            List!(Atom!("timestamp"), Atom!("2019-05-03 12:00:00-04:00")),
            List!(
                Atom!("payload"),
                List!(
                    Atom!("Add order"),
                    List!(
                        List!(Atom!("symbol_id"), Atom!("1")),
                        List!(Atom!("order_id"), Atom!("1")),
                        List!(Atom!("dir"), Atom!("buy")),
                        List!(Atom!("price"), Atom!("10.00")),
                        List!(Atom!("size"), Atom!("1")),
                        List!(Atom!("is_active"), Atom!("true"))
                    )
                )
            )
        );
        println!("{}", longer_sexp);
        // Expected:
        // (((timestamp 2019-05-03 12:00:00-04:00)
        //    (payload
        //      (Add order
        //        ((symbol_id 1)
        //          (order_id 1) (dir buy) (price 10.00) (size 1) (is_active true))))))
    }
}
