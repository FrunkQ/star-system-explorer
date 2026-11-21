//! The constants table's `derived_from` formulas (MP1c): products, quotients,
//! powers, sums and brackets of constant ids, numbers and `pi`. Juxtaposition
//! multiplies (`2 pi^5 k_B^4 / (15 h^3 c^2)`). Evaluation is plain f64
//! arithmetic with `sse_core::math::pow`, so every device gets the same bits.

use sse_core::math as m;

#[derive(Clone, PartialEq, Debug)]
enum Tok {
    Num(f64),
    Id(String),
    Op(char),
}

fn lex(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut out = vec![];
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch.is_ascii_digit() || ch == '.' {
            let st = i;
            while i < c.len()
                && (c[i].is_ascii_digit()
                    || c[i] == '.'
                    || ((c[i] == 'e' || c[i] == 'E')
                        && i + 1 < c.len()
                        && (c[i + 1].is_ascii_digit() || c[i + 1] == '-' || c[i + 1] == '+'))
                    || ((c[i] == '-' || c[i] == '+')
                        && i > st
                        && (c[i - 1] == 'e' || c[i - 1] == 'E')))
            {
                i += 1;
            }
            let t: String = c[st..i].iter().collect();
            out.push(Tok::Num(t.parse().map_err(|_| format!("bad number {t}"))?));
        } else if ch.is_alphabetic() || ch == '_' {
            let st = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(Tok::Id(c[st..i].iter().collect()));
        } else if "+-*/^()".contains(ch) {
            out.push(Tok::Op(ch));
            i += 1;
        } else {
            return Err(format!("unexpected {ch:?}"));
        }
    }
    Ok(out)
}

struct P<'a, F: FnMut(&str) -> Option<f64>> {
    t: Vec<Tok>,
    i: usize,
    look: &'a mut F,
}

impl<F: FnMut(&str) -> Option<f64>> P<'_, F> {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn expr(&mut self) -> Result<f64, String> {
        let mut v = self.term()?;
        while let Some(Tok::Op(o @ ('+' | '-'))) = self.peek().cloned() {
            self.i += 1;
            let r = self.term()?;
            v = if o == '+' { v + r } else { v - r };
        }
        Ok(v)
    }
    fn term(&mut self) -> Result<f64, String> {
        let mut v = self.factor()?;
        loop {
            match self.peek().cloned() {
                Some(Tok::Op('*')) => {
                    self.i += 1;
                    v *= self.factor()?;
                }
                Some(Tok::Op('/')) => {
                    self.i += 1;
                    v /= self.factor()?;
                }
                Some(Tok::Num(_)) | Some(Tok::Id(_)) | Some(Tok::Op('(')) => v *= self.factor()?,
                _ => return Ok(v),
            }
        }
    }
    fn factor(&mut self) -> Result<f64, String> {
        let b = self.unary()?;
        if let Some(Tok::Op('^')) = self.peek() {
            self.i += 1;
            let e = self.factor()?;
            return Ok(m::pow(b, e));
        }
        Ok(b)
    }
    fn unary(&mut self) -> Result<f64, String> {
        match self.peek().cloned() {
            Some(Tok::Op('-')) => {
                self.i += 1;
                Ok(-self.unary()?)
            }
            Some(Tok::Num(n)) => {
                self.i += 1;
                Ok(n)
            }
            Some(Tok::Id(id)) => {
                self.i += 1;
                if id == "pi" {
                    return Ok(m::PI);
                }
                (self.look)(&id).ok_or_else(|| format!("names {id}, which is not a constant"))
            }
            Some(Tok::Op('(')) => {
                self.i += 1;
                let v = self.expr()?;
                if self.peek() != Some(&Tok::Op(')')) {
                    return Err("a bracket is not closed".into());
                }
                self.i += 1;
                Ok(v)
            }
            t => Err(format!("unexpected {t:?}")),
        }
    }
}

/// Evaluate `s`, looking constant ids up with `look`.
pub fn eval(s: &str, mut look: impl FnMut(&str) -> Option<f64>) -> Result<f64, String> {
    let t = lex(s)?;
    let mut p = P {
        t,
        i: 0,
        look: &mut look,
    };
    let v = p.expr()?;
    if p.i != p.t.len() {
        return Err(format!("unexpected {:?}", p.t[p.i]));
    }
    Ok(v)
}

/// The constant ids a formula names.
pub fn names(s: &str) -> Vec<String> {
    let mut v: Vec<String> = lex(s)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|t| match t {
            Tok::Id(i) if i != "pi" => Some(i),
            _ => None,
        })
        .collect();
    v.sort();
    v.dedup();
    v
}
