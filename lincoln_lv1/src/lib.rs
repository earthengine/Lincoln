#![deny(bare_trait_objects)]

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use std::collections::HashMap;
use lincoln_common::traits::{StringLike, Access};
use serde::{Serialize, Deserialize};

quick_error! {
    #[derive(Debug)]
    pub enum LincolnLv1Error {
        VariableNotInScope
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Entry {
    name: String,
    block: Block,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Assignment {
    var: String,
    body: Body,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Body {
    Simple{
        block: Block
    },
    Compond{
        variants: Vec<Entry>
    },
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Call {
    callee: String,
    #[serde(skip_serializing_if="Option::is_none")]
    variant: Option<String>,
    args: Vec<String>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Block {
    args: Vec<String>,
    assignments: Vec<Assignment>,
    call: Call
}
impl Call {
    fn new<S>(callee: impl StringLike, variant: Option<impl StringLike>, args: impl IntoIterator<Item=S>) -> Self
        where S: StringLike
    {
        Call { callee: callee.to_string(), variant: variant.map(|s|s.to_string()), args: args.into_iter().map(StringLike::to_string).collect() }
    }
}
impl Block {
    fn new<S>(args: impl IntoIterator<Item=S>, call: Call) -> Self
        where S: StringLike
    {
        Block{args: args.into_iter().map(StringLike::to_string).collect(), assignments: vec![], call}
    }
    fn add_assignment(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }
}
impl Assignment {
    fn simple(var: impl StringLike, block: Block) -> Self {
        Assignment{var: var.to_string(), body: Body::Simple{block} }
    }
    fn compond(var: impl StringLike) -> Self {
        Assignment{ var: var.to_string(), body: Body::Compond{variants: vec![]} }
    }
    fn add_entry(&mut self, entry: Entry) {
        match &mut self.body {
            Body::Simple{block} => {
                self.body = Body::Compond{ variants: vec![
                    Entry{name: "default".into(), block:block.clone()},
                    entry
                ]};
            },
            Body::Compond{variants} => {
                variants.push(entry);
            }
        }
    }
}

fn get_call(names: &[&str], variant: Option<&str>) -> Call {
    Call::new(names[0], variant, names[1..].into_iter().map(|s|*s))
}
fn get_block(args: &[&str], call: Call) ->  Block {
    Block::new(args.into_iter().map(|s|*s), call)
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_call() {
        let c = get_call(&["c", "o"], None);
        assert_eq!(serde_json::to_string(&c).unwrap(), "{\"callee\":\"c\",\"args\":[\"o\"]}");
    }
    #[test]
    fn test_block() {
        let c = get_call(&["c", "n"], None);
        let b = get_block(&[],c);
        assert_eq!(serde_json::to_string(&b).unwrap(), "{\"args\":[],\"assignments\":[],\"call\":{\"callee\":\"c\",\"args\":[\"n\"]}}");
    }
    #[test]
    fn test_1() {
        let c = get_call(&["c", "o"], None);
        let b = get_block(&[], c);
        let a = Assignment::simple("c", b);
        let c = get_call(&["f", "c"], Some("drop"));
        let mut b = get_block(&[], c);
        b.add_assignment(a);
        assert_eq!(serde_json::to_string(&b).unwrap(), "{\"args\":[],\"assignments\":[],\"call\":{\"callee\":\"c\",\"args\":[\"n\"]}}");
    }
}
