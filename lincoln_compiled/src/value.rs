use crate::coderef::{CodeRef, GroupRef};
use crate::entries::{EvalFn, ExternEntry};
use crate::program::Program;
use crate::permutation::Permutation;
use lincoln_common::traits::{Access, AnyDebug};
use core::fmt::Debug;
use failure::Error;
use regex::Regex;
use smallvec::SmallVec;

use std::any::Any;

#[derive(Default)]
pub struct Context(Vec<Value>);
impl std::fmt::Debug for Context {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "![")?;
        let mut it = self.0.iter();
        if let Some(value) = it.next() {
            write!(fmt, "{:?}", value)?;
        }
        for value in it {
            write!(fmt, ", {:?}", value)?;
        }
        write!(fmt, "]!")
    }
}
impl Context {
    pub fn expect_args(&self, args: u8) -> Result<(), Error> {
        if self.len() != args {
            bail!(
                "Wrong number of arguments, need {} given {}",
                args,
                self.len()
            )
        } else {
            Ok(())
        }
    }
    pub fn append(self: &mut Self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
    pub fn len(&self) -> u8 {
        self.0.len() as u8
    }
    pub fn split(mut self, at: u8) -> (Self, Self) {
        let r = self.0.split_off(at as usize);
        let s0 = std::mem::replace(&mut self.0, vec![]);
        let ctx1 = Context(s0);
        let ctx2 = Context(r);
        (ctx1, ctx2)
    }
    pub fn push(self: &mut Self, v: Value) {
        self.0.push(v)
    }
    pub fn pop(&mut self) -> Result<Value, Error> {
        self.0.pop().ok_or(format_err!("Pop from empty"))
    }
    pub fn permutate(&mut self, p: Permutation) {
        p.permutate(&mut self.0)
    }
}
impl Drop for Context {
    fn drop(&mut self) {}
}

pub enum Value {
    Closure(SmallVec<[CodeRef; 5]>, Context),
    Wrapped(Box<dyn AnyDebug>),
    FinalReceiver(EvalFn),
}
impl std::fmt::Debug for Value {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Closure(grp, ctx) => write!(fmt, "{{{:?} {:?}}}", grp, ctx),
            Value::Wrapped(bx) => write!(fmt, "|{:?}|", bx),
            Value::FinalReceiver(_) => write!(fmt, "⟂"),
        }
    }
}
impl Value {
    pub fn wrap(v: impl Any + Debug + 'static) -> Self {
        Value::Wrapped(Box::new(v))
    }
    pub fn unwrap<T>(self, p: &Program) -> Result<T, Error>
    where
        T: 'static,
    {
        match self {
            Value::Closure(gr, ctx) => {
                if ctx.len() > 0 {
                    bail!("unwrap non-empty closure {:?} {:?}", gr, ctx);
                }
                if gr.len() != 1 {
                    bail!("unwrap multiple or none closure")
                }
                match gr[0] {
                    CodeRef::Extern(ext) => {
                        if let Some(ext) = ext.access(p) {
                            match ext {
                                ExternEntry::Value { value, .. } => return value().unwrap(p),
                                _ => bail!("Not a value"),
                            }
                        };
                        bail!("Invalid extern")
                    }
                    _ => bail!("Not extern value"),
                }
            }
            Value::Wrapped(bv) => bv
                .into_boxed_any()
                .downcast()
                .map(|v| *v)
                .map_err(|_| format_err!("Type mismatch")),
            Value::FinalReceiver(_) => bail!("Unwrap final receiver"),
        }
    }
    pub fn closure_prog(ent: GroupRef, ctx: Context, prog: &Program) -> Result<Value, Error> {
        Ok(Value::Closure(ent.get_vec(prog)?, ctx))
    }
    pub fn closure(ents: impl AsRef<[CodeRef]>, ctx: Context) -> Value {
        let mut ent = smallvec![];
        ent.extend_from_slice(ents.as_ref());
        Value::Closure(ent, ctx)
    }
    pub fn eval<'a>(
        self,
        p: &Program,
        mut ctx: Context,
        variant: u8,
    ) -> Result<(CodeRef, Context), Error> {
        debug!("eval_value({:?}) {:?}", &self, ctx);
        match self {
            Value::Closure(gr, mut ctx1) => {
                ctx.append(&mut ctx1);
                if variant as usize >= gr.len() {
                    bail!("variant out of bound {}/{}", variant, gr.len())
                }
                Ok((gr[variant as usize], ctx))
            }
            Value::FinalReceiver(f) => {
                let (_, ctx) = f(p, ctx)?;
                Ok((CodeRef::Termination, ctx))
            }
            _ => bail!("Not a closure"),
        }
    }
    pub fn parse_string(s: impl AsRef<str>) -> Result<SmallVec<[Self; 5]>, Error> {
        let reg = Regex::new(",")?;
        let us = Regex::new("(?P<value>[1-9]?[0-9]*|0)usize")?;
        let mut r = smallvec![];
        for m in reg.split(s.as_ref()) {
            if let Some(capture) = us.captures(m) {
                if let Some(value) = capture.name("value") {
                    r.push(Value::wrap(value.as_str().parse::<usize>()?))
                }
            }
        }
        Ok(r)
    }
}
