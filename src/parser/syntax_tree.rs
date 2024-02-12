// Syntax tree structs (Tokens structures)

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// Expression type
#[derive(Debug, Clone)]
pub enum ExpressionTag {
    // Empty string expression
    EmptyExpression,

    // Dot expression `.`
    DotExpression,

    // Character expression
    CharacterExpression { value: char },

    // A grouped expression (...)
    // where `...` is another regular expression
    Group,
}

// (Wrapper) Expression objects after parsing
#[derive(Debug)]
pub struct Regexp {
    // -- Which expression this wrapper contains
    pub tag: ExpressionTag,

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
        let parent = None;
        let children = RefCell::new(vec![]);
        Regexp {
            tag,
            parent,
            children,
        }
    }
}
