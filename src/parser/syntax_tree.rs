// Syntax tree structs

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// Expression type
#[derive(Debug, Clone)]
pub enum ExpressionTag {
    // Empty string expression
    EmptyExpression,
}

// (Wrapper) Expression objects after parsing
#[derive(Debug)]
pub struct Regexp {
    // -- Which expression this wrapper contains
    pub tag: ExpressionTag,

    // -- Parent expression of `tag`
    // * We use a Weak reference to avoid reference cycles
    // because parent points to child and child points to parent
    // thus reference count of either would never be zero
    // and hence they will never be droped
    // * Field `parent` is Option because syntax tree root has no parent
    // * We use a RefCell to allow interior mutability in case
    // `parent` needed to modify is data
    pub parent: Option<Weak<RefCell<Regexp>>>,

    // -- Children expressions of `tag`
    // * We use RefCell to allow interior mutability
    // in case we needed to modify Vec `children`
    // or a particular child needs to be modified
    // * We use Rc because Expressions are always shared
    // there is no object which owns an `Regexp`
    // * We use RefCell<Regexp> to allow interior mutability
    // in case an `Regexp` needs to be modified
    pub children: RefCell<Vec<Rc<RefCell<Regexp>>>>,
}

// while initializing `Regexp` objects
// we need some intricate initializations
// in which we need a `default Regexp` object
impl Default for Regexp {
    fn default() -> Self {
        Regexp {
            // Assume this exprssion represents the empty regular expression
            tag: ExpressionTag::EmptyExpression,
            parent: None,
            children: RefCell::new(vec![]),
        }
    }
}
