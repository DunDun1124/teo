use crate::core::field::builder::FieldBuilder;


use crate::parser::ast::argument::Argument;
use crate::prelude::Value;

pub(crate) fn write_if_decorator(args: Vec<Argument>, field: &mut FieldBuilder) {
    match args.get(0).unwrap().resolved.as_ref().unwrap().as_value().unwrap() {
        Value::Pipeline(p) => {
            field.write_if(p.clone());
        }
        _ => panic!("Wrong argument passed to writeIf.")
    }
}
