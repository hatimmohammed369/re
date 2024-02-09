// Parser module
pub mod syntax_tree;

use crate::scanner::{Scanner, tokens::*};

pub struct Parser {
    scanner: Scanner,
    current: Option<Token>,
}
