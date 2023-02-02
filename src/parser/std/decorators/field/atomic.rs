use crate::core::field::builder::FieldBuilder;

use crate::parser::ast::argument::Argument;

pub(crate) fn atomic_decorator(_args: Vec<Argument>, field: &mut FieldBuilder) {
    field.atomic = true;
}
