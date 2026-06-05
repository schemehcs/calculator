/*
S: TE
E: (+|-)TE | nil
T: FG
G: (*|/)FG | nil
F: N | '('S')'
*/

use bigdecimal::BigDecimal;
use std::{
    iter::Peekable,
    ops::{Add, Div, Mul, Sub},
    str::{CharIndices, FromStr},
};

#[derive(Debug)]
pub struct S {
    pub t: T,
    pub e: E,
}

impl S {
    fn new(t: T, e: E) -> Self {
        Self { t, e }
    }

    fn value(&self) -> BigDecimal {
        let lv = self.t.value();
        self.e.value(lv)
    }
}

#[derive(Debug)]
pub struct T {
    pub f: F,
    pub g: G,
}

impl T {
    fn new(f: F, g: G) -> Self {
        Self { f, g }
    }

    fn value(&self) -> BigDecimal {
        let lv = self.f.value();
        self.g.value(lv)
    }
}

#[derive(Debug)]
pub enum Eop {
    Add,
    Sub,
}

#[derive(Debug)]
pub enum Gop {
    Mul,
    Div,
}

#[derive(Debug)]
pub enum E {
    Some { op: Eop, t: T, e: Box<E> },
    None,
}

impl E {
    pub fn new(op: Eop, t: T, e: E) -> Self {
        Self::Some {
            op,
            t,
            e: Box::new(e),
        }
    }

    pub fn value(&self, lv: BigDecimal) -> BigDecimal {
        match self {
            E::Some { op, t, e } => {
                let bdop = match op {
                    Eop::Add => BigDecimal::add,
                    Eop::Sub => BigDecimal::sub,
                };
                let rv = t.value();
                let ev = bdop(lv, rv);
                e.value(ev)
            }
            E::None => lv,
        }
    }
}

#[derive(Debug)]
pub enum G {
    Some { op: Gop, f: F, g: Box<G> },
    None,
}

impl G {
    pub fn new(op: Gop, f: F, g: G) -> Self {
        Self::Some {
            op,
            f,
            g: Box::new(g),
        }
    }

    pub fn value(&self, lv: BigDecimal) -> BigDecimal {
        match self {
            G::Some { op, f, g } => {
                let bdop = match op {
                    Gop::Mul => BigDecimal::mul,
                    Gop::Div => BigDecimal::div,
                };
                let rv = f.value();
                let gv = bdop(lv, rv);
                g.value(gv)
            }
            G::None => lv,
        }
    }
}

#[derive(Debug)]
pub enum F {
    Num(BigDecimal),
    S(Box<S>),
}

impl F {
    pub fn value(&self) -> BigDecimal {
        match self {
            F::Num(e) => e.clone(),
            F::S(b) => b.value(),
        }
    }
}

struct Cursor<'a> {
    inner: Peekable<CharIndices<'a>>,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            inner: input.char_indices().peekable(),
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<(usize, char)> {
        self.inner.peek().copied()
    }

    #[inline]
    fn next(&mut self) -> Option<(usize, char)> {
        self.inner.next()
    }

    fn consume<F: Fn(char) -> bool>(&mut self, f: F) {
        while let Some((_, c)) = self.peek() {
            if f(c) {
                self.next();
            } else {
                return;
            }
        }
    }

    fn consume_ws(&mut self) {
        self.consume(char::is_whitespace);
    }

    fn expect(&mut self, ec: char) -> anyhow::Result<()> {
        if let Some((i, c)) = self.next() {
            if c != ec {
                anyhow::bail!("expect char:{}, but found {}:{} at index", ec, i, c);
            } else {
                Ok(())
            }
        } else {
            anyhow::bail!("expect char:{}, but reached end", ec);
        }
    }
}

pub struct Parser<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }

    pub fn parse(&mut self) -> anyhow::Result<S> {
        let s = self.parse_s()?;
        self.cursor.consume_ws();
        if let Some((i, c)) = self.cursor.peek() {
            anyhow::bail!("expect end of input but found {}:{}", i, c);
        } else {
            Ok(s)
        }
    }

    fn parse_s(&mut self) -> anyhow::Result<S> {
        let t = self.parse_t()?;
        let e = self.parse_e()?;
        Ok(S::new(t, e))
    }

    fn parse_t(&mut self) -> anyhow::Result<T> {
        let f = self.parse_f()?;
        let g = self.parse_g()?;
        Ok(T::new(f, g))
    }

    fn parse_e(&mut self) -> anyhow::Result<E> {
        if let Some((_, c)) = self.cursor.peek() {
            let op = match c {
                '+' => {
                    self.cursor.next();
                    Eop::Add
                }
                '-' => {
                    self.cursor.next();
                    Eop::Sub
                }
                _ => return Ok(E::None),
            };
            let t = self.parse_t()?;
            let e = self.parse_e()?;
            Ok(E::new(op, t, e))
        } else {
            Ok(E::None)
        }
    }

    fn parse_g(&mut self) -> anyhow::Result<G> {
        self.cursor.consume_ws();
        if let Some((_, c)) = self.cursor.peek() {
            let op = match c {
                '*' => {
                    self.cursor.next();
                    Gop::Mul
                }
                '/' => {
                    self.cursor.next();
                    Gop::Div
                }
                _ => return Ok(G::None),
            };
            let f = self.parse_f()?;
            let g = self.parse_g()?;
            Ok(G::new(op, f, g))
        } else {
            Ok(G::None)
        }
    }

    fn parse_f(&mut self) -> anyhow::Result<F> {
        self.cursor.consume_ws();
        if let Some((i, c)) = self.cursor.next() {
            match c {
                '0'..='9' | '+' | '-' => Ok(F::Num(self.parse_num(c)?)),
                '(' => {
                    let s = self.parse_s()?;
                    self.cursor.consume_ws();
                    self.cursor.expect(')')?;
                    Ok(F::S(Box::new(s)))
                }
                _ => anyhow::bail!("invalid charactor {}:{}", i, c),
            }
        } else {
            anyhow::bail!("expect F but reached EOI");
        }
    }

    fn parse_num(&mut self, leading: char) -> anyhow::Result<BigDecimal> {
        let mut buf = String::new();
        buf.push(leading);
        let mut decimal = false;
        while let Some((i, c)) = self.cursor.peek() {
            match c {
                '0'..='9' => buf.push(c),
                '.' => {
                    if !decimal {
                        buf.push(c);
                        decimal = true;
                    } else {
                        anyhow::bail!("invalid num {}:{}", i, c);
                    }
                }
                _ => break,
            }
            self.cursor.next();
        }
        if buf.ends_with('.') {
            anyhow::bail!("invalid number {}", buf);
        }
        Ok(BigDecimal::from_str(&buf)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let expr = "2.000001";
        let mut parser = Parser::new(expr);
        let ast = parser.parse().unwrap();
        assert_eq!(BigDecimal::from_str(expr).unwrap(), ast.value());
    }

    #[test]
    fn test_composed() {
        let expr = "1 +(2+ (3 + ( (4* 5 ) /2) )) *3";
        let mut parser = Parser::new(expr);
        let ast = parser.parse().unwrap();
        assert_eq!(BigDecimal::from_str("46").unwrap(), ast.value());
    }
}
