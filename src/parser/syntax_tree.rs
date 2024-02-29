// Syntax tree structs (Tokens structures)

use crate::scanner::tokens::*;
use std::cell::RefCell;
use std::collections::LinkedList;
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

    pub fn deep_copy(&self) -> Regexp {
        // Copy expressions one level at a time
        // Each a time iterate through children in order from first to last,
        // copy the child itself then store its children for next iteration
        // this way children of current expressions are also ordered
        // because we iterate through children in order too
        // repeat until all children in source Regexp have been copied
        // in other words all levels copied, we copied last level
        // which its expressions have no children
        let mut source_children = LinkedList::from([Rc::new(RefCell::new(self.clone()))]);
        let deep_copy = Rc::new(RefCell::new(Regexp {
            tag: self.tag,
            pattern: self.pattern.clone(),
            parent: None,
            children: RefCell::new(vec![]),
        }));
        let mut dest_children = LinkedList::from([Rc::clone(&deep_copy)]);

        while !source_children.is_empty() {
            let source_level_end = source_children.len();
            for _ in 1..=source_level_end {
                let source_child = source_children.pop_front().unwrap();
                let source_child = RefCell::borrow(&source_child);

                let source_child_offspring = source_child.children.borrow();
                let source_child_offspring = source_child_offspring.iter().map(|kid| {
                    source_children.push_back(Rc::clone(kid));
                    RefCell::borrow(kid)
                });

                let dest_child = dest_children.pop_front().unwrap();
                let dest_child_offspring = RefCell::borrow_mut(&dest_child);
                let dest_child_offspring = &mut dest_child_offspring.children.borrow_mut();

                for src_kid in source_child_offspring {
                    let new_dest_child = {
                        let tag = src_kid.tag;
                        let parent = Some(Rc::downgrade(&dest_child));
                        let pattern = src_kid.pattern.clone();
                        let children = RefCell::new(vec![]);

                        Rc::new(RefCell::new(Regexp {
                            tag,
                            parent,
                            pattern,
                            children,
                        }))
                    };

                    dest_children.push_back(Rc::clone(&new_dest_child));
                    dest_child_offspring.push(new_dest_child);
                }
            }
        }

        let deep_copy = RefCell::borrow(&deep_copy).clone();
        deep_copy
    }
}

impl Clone for Regexp {
    fn clone(&self) -> Self {
        let tag = self.tag;
        let pattern = self.pattern.clone();
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
