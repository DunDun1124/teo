use crate::parser::ast::span::Span;
use crate::parser::ast::item::Item;
use crate::parser::ast::identifier::Identifier;
use crate::prelude::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) id: usize,
    pub(crate) source_id: usize,
    pub(crate) items: Vec<Item>,
    pub(crate) span: Span,
    pub(crate) bind: Option<(String, u16)>,
    pub(crate) jwt_secret: Option<String>,
    pub(crate) path_prefix: Option<String>,
}

impl Config {
    pub(crate) fn new(item_id: usize, source_id: usize, items: Vec<Item>, span: Span) -> Self {
        Self {
            id: item_id,
            source_id,
            items,
            span,
            bind: None,
            jwt_secret: None,
            path_prefix: None,
        }
    }
}
