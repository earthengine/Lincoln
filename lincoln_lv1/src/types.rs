use lincoln_common::traits::StringLike;
use std::fmt;

///
///  A Tuple is a group of types. Then 
/// 
#[derive(Serialize, Deserialize)]
pub struct TupleType {
    elements: Vec<LCType>,
}
#[derive(Serialize, Deserialize)]
pub struct LCType {
    name: Option<String>,
    variants: Vec<TupleType>,
}
impl fmt::Display for LCType {
    fn fmt(&self, fmt:&mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            if self.variants.len()==0 {
                write!(fmt, "{}", name)
            } else {
                write!(fmt, "{}:~{{{}}}", name, self.variants
                                                     .iter()
                                                     .map(|v| format!("{}",v))
                                                     .collect::<Vec<String>>()
                                                     .join("; "))
            }

        } else if self.variants.len()==1 {
            write!(fmt,"~{}", self.variants[0])
        } else {
            write!(fmt, "~{{{}}}", self.variants
                                       .iter()
                                       .map(|v| format!("{}",v))
                                       .collect::<Vec<String>>()
                                       .join("; "))

        }
    }
}
impl fmt::Debug for LCType {
    fn fmt(&self, fmt:&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl PartialEq<LCType> for LCType {
    fn eq(&self, other: &LCType) -> bool {
        self.name == other.name &&
        self.variants.len() == other.variants.len() &&
        self.variants.iter().zip(other.variants.iter()).all(|(v1,v2)| v1==v2)
    }
}
impl fmt::Display for TupleType {
    fn fmt(&self, fmt:&mut fmt::Formatter) -> fmt::Result { 
        if self.elements.len()==1 {
            write!(fmt, "{}", self.elements[0])
        } else {
            write!(fmt, "({})", self.elements
                                    .iter()
                                    .map(|v| format!("{}",v))
                                    .collect::<Vec<String>>()
                                    .join(", "))
        }
    }
}
impl fmt::Debug for TupleType {
    fn fmt(&self, fmt:&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl PartialEq<TupleType> for TupleType {
    fn eq(&self, other: &TupleType) -> bool {
        self.elements.len()==other.elements.len() &&
        self.elements.iter().zip(other.elements.iter()).all(|(v1,v2)| v1==v2)
    }
}

impl LCType {
    pub fn atom(name: impl StringLike) -> Self {
        Self::named(name, vec![])
    }
    pub fn named(name: impl StringLike, variants: Vec<TupleType>) -> Self {
        LCType {
            name: Some(name.to_string()),
            variants: variants,
        }
    }
    pub fn unnamed(variants: Vec<TupleType>) -> Self {
        LCType {
            name: None,
            variants: variants,
        }
    }
    pub fn negation(t: LCType) -> Self {
        let t = TupleType::new(vec![t]);
        Self::unnamed(vec![t])
    }
    pub fn bottom() -> Self {
        let u = TupleType::unit();
        Self::unnamed(vec![u])
    }
    pub fn empty() -> Self {
        Self::unnamed(vec![])
    }
    pub fn ntuple(elements: Vec<LCType>) -> Self {
        Self::unnamed(vec![TupleType::new(elements)])
    }
}
impl TupleType {
    pub fn new(elements: Vec<LCType>) -> Self {
        TupleType {
            elements
        }
    }
    pub fn unit() -> Self {
        Self::new(vec![])        
    }
    pub fn negation(t:LCType) -> Self {
        let t = LCType::negation(t);
        TupleType::new(vec![t])
    }
    pub fn ntuple(elements: Vec<LCType>) -> Self {
        Self::new(vec![LCType::ntuple(elements)])
    }
}


#[cfg(test)]
mod tests {
    use crate::types::{LCType, TupleType};
    #[test]
    fn test_atom() {
        //Atom types are NOptionType with a name and zero variants
        let t = LCType::atom("name");
        assert_eq!(format!("{}", t), "name")
    }
    #[test]
    fn test_unit() {
        //Bottom type is a tuple without elements        
        assert_eq!(format!("{}", TupleType::unit()), "()")
    }
    #[test]
    fn test_bottom() {
        //Bottom type is the negation of a unit
        assert_eq!(format!("{}", LCType::bottom()), "~()")
    }    #[test]
    fn test_empty() {
        //Zero type is an noption without variants
        assert_eq!(format!("{}", LCType::empty()), "~{}")
    }
    #[test]
    fn test_top() {
        //Top type is a type that receives a zero type
        assert_eq!(format!("{}", LCType::negation(LCType::empty())), "~~{}")
    }
    #[test]
    fn test_bool() {
        //Bool type receives a type that both variants are unit
        let t = LCType::unnamed(vec![TupleType::unit(),TupleType::unit()]);
        let t = LCType::negation(t);
        assert_eq!(format!("{}", t), "~~{(); ()}")
    }
    #[test]
    fn test_droppable() {
        //A droppable type is a type with two variants: claim the inner type, or drop
        let b = TupleType::new(vec![LCType::bottom()]);
        let t = LCType::unnamed(vec![TupleType::negation(LCType::atom("T")),b]);
        assert_eq!(format!("{}", t), "~{~T; ~()}")
    }
    #[test]
    fn test_copiable() {
        //A copiable type is a type with two variants: clain the inner type, or copy itself
        //This is the first example of a recursive type: it have a name to be refered
        //inside the definition.
        let c = TupleType::ntuple(vec![LCType::atom("S"),LCType::atom("S")]);
        let t = TupleType::new(vec![LCType::negation(LCType::atom("T"))]);
        let t = LCType::named("S", vec![t, c]);
        assert_eq!(format!("{}", t), "S:~{~T; ~(S, S)}")
    }
    #[test]
    fn test_equal() {
        let a = LCType::atom("A");
        let b = LCType::atom("B");
        assert_ne!(a, b);
        let a = LCType::bottom();
        assert_ne!(a, b);
        let b = LCType::empty();
        assert_ne!(a, b);
    }
}