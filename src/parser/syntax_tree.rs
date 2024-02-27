// Syntax tree structs (Tokens structures)

use crate::scanner::tokens::*;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone, Copy)]
pub enum Quantifier {
    None,       // No quantifier
    ZeroOrOne,  // Quantifier ?
    ZeroOrMore, // Quantifier *
    OneOrMore,  // Quantifier +
}

impl From<&Option<Token>> for Quantifier {
    fn from(token: &Option<Token>) -> Self {
        match token {
            Some(tok) => {
                // I do not want `cargo fmt` remove the outer block
                match tok.name {
                    TokenName::Mark => Quantifier::ZeroOrOne,
                    TokenName::Star => Quantifier::ZeroOrMore,
                    TokenName::Plus => Quantifier::OneOrMore,
                    _ => Quantifier::None,
                }
            }
            None => Quantifier::None,
        }
    }
}

impl Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_value = match self {
            Self::None => "",
            Self::ZeroOrOne => "?",
            Self::ZeroOrMore => "*",
            Self::OneOrMore => "+",
        };
        write!(f, "{string_value}")
    }
}
// Expression type
#[derive(Debug, Clone, Copy)]
pub enum ExpressionTag {
    // Empty string expression
    // the expression between ( and ) in string `()`
    // or between ( and | and ) in string `(|)
    // or around | in string `|` or string `||`
    EmptyExpression,

    // Character expression, like `z`
    CharacterExpression {
        // If field `value` is Option::<char>::None
        // then this CharacterExpression expression is actually a dot expression
        // . \ .? \ .* \ .+
        value: Option<char>,
        quantifier: Quantifier,
    },

    // Concatenation expression
    // something like `a.b.c(abc)`
    Concatenation,

    // Alternation expression
    // something like (a|bc|x.y.z)
    Alternation,

    // A grouped expression (...)
    // where `...` is another regular expression
    Group {
        quantifier: Quantifier,
    },
}

// (Wrapper) Expression objects after parsing
#[derive(Debug)]
pub struct Regexp {
    // -- Which expression this wrapper contains
    pub tag: ExpressionTag,

    // Pattern of this (sub)expression
    pub pattern: String,

    // -- Parent expression of this object
    // * We use a Weak reference to avoid reference cycles
    // because parent points to child and child points to parent
    // thus reference count of either would never be zero
    // and hence they will never be droped
    // * Field `parent` is Option because syntax tree root has no parent
    // * We use a RefCell to allow interior mutability in case
    // `parent` needed to modify is data
    pub parent: Option<Weak<RefCell<Regexp>>>,

    // -- Children expressions of this object
    // * We use RefCell to allow interior mutability
    // in case we needed to modify Vec `children`
    // or a particular child needs to be modified
    // * We use Rc because Expressions are always shared
    // there is no object which owns an `Regexp`
    // * We use RefCell<Regexp> to allow interior mutability
    // in case an `Regexp` needs to be modified
    pub children: RefCell<Vec<Rc<RefCell<Regexp>>>>,
}

// Replace `Default` trait with a constructor which at least initializes
// the object tag field rather than using `ExpressionTag::EmptyExpression`
// as a default tag, and still gives an `initialized` object
impl Regexp {
    pub fn new(tag: ExpressionTag) -> Self {
        let pattern = String::new();
        let parent = None;
        let children = RefCell::new(vec![]);
        Regexp {
            tag,
            pattern,
            parent,
            children,
        }
    }
}

impl Clone for Regexp {
    fn clone(&self) -> Self {
        let tag = self.tag;
        let pattern = String::new();
        let parent = self.parent.as_ref().map(Weak::clone);
        let children = RefCell::new(self.children.borrow().iter().map(Rc::clone).collect());
        Regexp {
            tag,
            pattern,
            parent,
            children,
        }
    }
}

impl Display for Regexp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pattern)
    }
}
