// Syntax tree structs (Tokens structures)

use std::collections::LinkedList;
use std::fmt::Display;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug, Clone, Copy)]
pub enum Quantifier {
    None,       // No quantifier
    ZeroOrOne,  // Quantifier ?
    ZeroOrMore, // Quantifier *
    OneOrMore,  // Quantifier +
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

// Expression types
#[derive(Debug, Clone, Copy)]
pub enum ExpressionType {
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
pub struct ParsedRegexp {
    // -- Which expression this wrapper contains
    pub expression_type: ExpressionType,

    // Pattern of this (sub)expression
    pub pattern: Arc<str>,

    // -- Parent expression of this object
    // * We use a Weak reference to avoid reference cycles
    // because parent points to child and child points to parent
    // thus reference count of either would never be zero
    // and hence they will never be droped
    // * Field `parent` is Option because syntax tree root has no parent
    // * We use a RwLock to allow interior mutability in case
    // `parent` needed to modify is data
    pub parent: Option<Weak<RwLock<ParsedRegexp>>>,

    // -- Children expressions of this object
    // * We use RwLock to allow interior mutability
    // in case we needed to modify Vec `children`
    // or a particular child needs to be modified
    // * We use Arc because Expressions are always shared
    // there is no object which owns an `ParsedRegexp`
    // * We use RwLock<ParsedRegexp> to allow interior mutability
    // in case an `ParsedRegexp` needs to be modified
    pub children: RwLock<Vec<Arc<RwLock<ParsedRegexp>>>>,
}

// Replace `Default` trait with a constructor which at least initializes
// the object tag field rather than using `ExpressionTag::EmptyExpression`
// as a default tag, and still gives an `initialized` object
impl ParsedRegexp {
    pub fn new(expr_type: ExpressionType) -> Self {
        ParsedRegexp {
            expression_type: expr_type,
            pattern: Arc::from(""),
            parent: None,
            children: RwLock::new(vec![]),
        }
    }

    pub fn debug_as_strings(&self) -> String {
        let mut debug = String::new();
        debug.push_str("ParsedRegexp {\n");
        let indent = "  "; // 2 spaces
        debug.push_str(&format!("{indent}pattern: {},\n", self.pattern));

        if let Some(parent) = &self.parent {
            let parent = parent.upgrade().unwrap();
            let parent = &parent.read().unwrap().pattern;
            debug.push_str(&format!("{indent}parent : {},\n", parent));
        }

        let children = self.children.read().unwrap();
        debug.push_str(&format!("{indent}children = {{"));
        if !children.is_empty() {
            debug.push('\n');
            for child in children.iter() {
                let child = &child.read().unwrap().pattern;
                debug.push_str(&format!("{indent}{indent}{child},\n"));
            }
        }
        debug.push_str(&format!("{indent}}},\n"));

        debug.push('}');
        debug
    }

    pub fn deep_copy(&self) -> Arc<RwLock<ParsedRegexp>> {
        // Copy expressions one level at a time
        // Each a time iterate through children in order from first to last,
        // copy the child itself then store its children for next iteration
        // this way children of current expressions are also ordered
        // because we iterate through children in order too
        // repeat until all children in source ParsedRegexp have been copied
        // in other words all levels copied, we copied last level
        // which its expressions have no children
        let mut source_children = LinkedList::from([Arc::new(RwLock::new(Self::clone(self)))]);
        let deep_copy = Arc::new(RwLock::new(ParsedRegexp {
            expression_type: self.expression_type,
            pattern: Arc::from(self.pattern.as_ref()),
            parent: None,
            children: RwLock::new(vec![]),
        }));
        let mut dest_children = LinkedList::from([Arc::clone(&deep_copy)]);

        while !source_children.is_empty() {
            let source_level_end = source_children.len();
            for _ in 1..=source_level_end {
                let source_child = source_children.pop_front().unwrap();
                let source_child = source_child.read().unwrap();

                let source_child_offspring = source_child.children.read().unwrap();
                let source_child_offspring = source_child_offspring.iter().map(|kid| {
                    source_children.push_back(Arc::clone(kid));
                    kid.read().unwrap()
                });

                let dest_child = dest_children.pop_front().unwrap();
                let dest_child_offspring = dest_child.write().unwrap();
                let mut dest_child_offspring = dest_child_offspring.children.write().unwrap();

                for src_kid in source_child_offspring {
                    let new_dest_child = Arc::new(RwLock::new(ParsedRegexp {
                        expression_type: src_kid.expression_type,
                        parent: Some(Arc::downgrade(&dest_child)),
                        pattern: Arc::from(src_kid.pattern.as_ref()),
                        children: RwLock::new(vec![]),
                    }));

                    dest_children.push_back(Arc::clone(&new_dest_child));
                    dest_child_offspring.push(new_dest_child);
                }
            }
        }

        deep_copy
    }
}

impl Clone for ParsedRegexp {
    fn clone(&self) -> Self {
        ParsedRegexp {
            expression_type: self.expression_type,
            pattern: Arc::from(self.pattern.as_ref()),
            parent: self.parent.as_ref().map(Weak::clone),
            children: RwLock::new(
                self.children
                    .read()
                    .unwrap()
                    .iter()
                    .map(Arc::clone)
                    .collect(),
            ),
        }
    }
}

impl Display for ParsedRegexp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pattern)
    }
}
