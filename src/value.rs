use crate::coderef::{Access, CodeRef, GroupRef};
use crate::permutation::Permutation;
use crate::program::EvalFn;
use crate::program::{ExternEntry, Program};
use failure::Error;

use std::any::Any;

pub struct Context(Vec<Value>);
impl std::fmt::Debug for Context {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "![")?;
        for value in self.0.iter() {
            write!(fmt, "{:?}", value)?;
        }
        write!(fmt, "]!")
    }
}
impl Context {
    pub fn new() -> Context {
        Context(vec![])
    }
    pub fn expect_args(&self, args: u8) -> Result<(), Error> {
        if self.len() != args {
            bail!(
                "Wrong number of arguments, given {} max {}",
                args,
                self.len()
            )
        } else {
            Ok(())
        }
    }
    pub fn pop_first(&mut self) -> Result<Value, Error> {
        if self.len() == 0 {
            bail!("Pop from empty")
        }
        Ok(self.0.swap_remove(0))
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
    Closure(GroupRef, Context),
    Wrapped(Box<dyn Any>),
    FinalReceiver(EvalFn),
}
impl std::fmt::Debug for Value {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Closure(grp, ctx) => write!(fmt, "{{{:?} {:?}}}", grp, ctx),
            Value::Wrapped(bx) => write!(fmt, "|{:?}|", bx),
            Value::FinalReceiver(f) => write!(fmt, "{:?}", f as *const _ as *const ()),
        }
    }
}
impl Value {
    pub fn wrap(v: impl Any + 'static) -> Self {
        Value::Wrapped(Box::new(v))
    }
    pub fn unwrap<T>(self, p: &Program) -> Result<T, Error>
    where
        T: 'static,
    {
        match self {
            Value::Closure(gr, ctx) => {
                if ctx.len() > 0 {
                    bail!("unwrap non-empty closure");
                }
                match gr.as_entry_ref(p, 0)? {
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
                .downcast()
                .map(|v| *v)
                .map_err(|_| format_err!("Type mismatch")),
            Value::FinalReceiver(_) => bail!("Unwrap final receiver"),
        }
    }
    pub fn closure(ent: GroupRef, ctx: Context) -> Value {
        Value::Closure(ent, ctx)
    }
    pub fn eval<'a>(
        self,
        p: &Program,
        mut ctx: Context,
        variant: u8,
    ) -> Result<(CodeRef, Context), Error> {
        match self {
            Value::Closure(gr, mut ctx1) => {
                ctx.append(&mut ctx1);
                let ent = gr.as_entry_ref(p, variant)?;
                Ok((ent, ctx))
            }
            Value::FinalReceiver(f) => {
                let (_, ctx) = f(p, ctx)?;
                Ok((CodeRef::Termination, ctx))
            }
            _ => bail!("Not a closure"),
        }
    }
}
