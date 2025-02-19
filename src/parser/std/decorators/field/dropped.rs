use crate::core::field::Field;
use crate::parser::ast::argument::Argument;

pub(crate) fn dropped_decorator(_args: Vec<Argument>, field: &mut Field) {
    field.dropped = true;
}
