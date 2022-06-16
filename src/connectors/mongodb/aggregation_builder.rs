use serde_json::{Value as JsonValue};
use bson::{Bson, doc, Document};
use crate::core::graph::Graph;
use crate::core::model::Model;
use crate::error::ActionError;


#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) enum QueryPipelineType {
    Unique,
    First,
    Many
}

fn build_where_input(
    model: &Model,
    graph: &Graph,
    r#where: Option<&JsonValue>,
) -> Result<Document, ActionError> {
    Ok(doc!{})
}

fn build_lookup_inputs(
    model: &Model,
    graph: &Graph,
    r#type: QueryPipelineType,
    mutation_mode: bool,
    include: &JsonValue,
) -> Result<Vec<Document>, ActionError> {
    let include = include.as_object();
    if include.is_none() {
        let model_name = &model.name;
        return Err(ActionError::invalid_query_input(format!("'include' on model '{model_name}' is not an object. Please check your input.")));
    }
    let include = include.unwrap();
    let mut retval: Vec<Document> = vec![];
    for (key, value) in include.iter() {
        let relation = model.relation(key);
        if relation.is_none() {
            let model_name = &model.name;
            return Err(ActionError::invalid_query_input(format!("Relation '{key}' on model '{model_name}' is not exist. Please check your input.")));
        }
        let relation = relation.unwrap();
        let relation_model_name = &relation.model;
        let relation_model = graph.model(relation_model_name);
        if value.is_boolean() || value.is_object() {
            let mut let_value = doc!{};
            let mut eq_values: Vec<Document> = vec![];
            for (index, field_name) in relation.fields.iter().enumerate() {
                let reference_name = relation.references.get(index).unwrap();
                let_value.insert(field_name, format!("${reference_name}"));
                eq_values.push(doc!{format!("${reference_name}"): format!("$${reference_name}")});
            }
            let mut inner_pipeline = if value.is_object() {
                build_query_pipeline_from_json(relation_model, graph, r#type, mutation_mode, value)?
            } else {
                vec![]
            };
            //
            // {
            //     "$match": {
            //     "$expr": {
            //         "$and": eq_values
            //     }
            // }
            // }
            let inner_match = inner_pipeline.iter().find(|v| v.get("$match").is_some());
            let has_inner_match = inner_match.is_some();
            let mut inner_match = if has_inner_match {
                inner_match.unwrap().clone()
            } else {
                doc!{"$match": {}}
            };
            if inner_match.get("$expr").is_none() {
                inner_match.insert("$expr", doc!{});
            }
            if inner_match.get("$expr").unwrap().as_document().unwrap().get("$and").is_none() {
                inner_match.get_mut("$expr").unwrap().as_document_mut().unwrap().insert("$and", vec![] as Vec<Document>);
            }
            inner_match.get_mut("$expr").unwrap().as_document_mut().unwrap().get_mut("$and").unwrap().as_array_mut().unwrap().extend(eq_values.iter().map(|item| Bson::Document(item.clone())));
                // push here
            let lookup = doc!{"$lookup": {
                "from": &relation_model.table_name,
                "as": key,
                "let": let_value,
                "pipeline": inner_pipeline
            }};
            retval.push(lookup);
        } else {
            let model_name = &model.name;
            return Err(ActionError::invalid_query_input(format!("Relation '{key}' on model '{model_name}' has a unrecognized value. It's either a boolean or an object. Please check your input.")));
        }
    }
    Ok(retval)
}

fn build_query_pipeline(
    model: &Model,
    graph: &Graph,
    r#type: QueryPipelineType,
    mutation_mode: bool,
    r#where: Option<&JsonValue>,
    order_by: Option<&JsonValue>,
    take: Option<usize>,
    skip: Option<usize>,
    page_size: Option<usize>,
    page_number: Option<usize>,
    include: Option<&JsonValue>,
    select: Option<&JsonValue>,
) -> Result<Vec<Document>, ActionError> {
    let mut retval: Vec<Document> = vec![];
    // $match
    let r#match = build_where_input(model, graph, r#where)?;
    if !r#match.is_empty() {
        retval.push(r#match);
    }
    // $sort

    // $skip and $limit
    if page_size.is_some() && page_number.is_some() {
        retval.push(doc!{"$skip": ((page_number.unwrap() - 1) * page_size.unwrap()) as i64});
        retval.push(doc!{"limit": page_size.unwrap() as i64});
    } else {
        if skip.is_some() {
            retval.push(doc!{"$skip": skip.unwrap() as i64});
        }
        if take.is_some() {
            retval.push(doc!{"$limit": skip.unwrap() as i64});
        }
    }
    // $project
    // $lookup
    if include.is_some() {
        let mut lookups = build_lookup_inputs(model, graph, r#type, mutation_mode, include.unwrap())?;
        if !lookups.is_empty() {
            retval.append(&mut lookups);
        }
    }

    Ok(retval)
}

fn build_query_pipeline_from_json(
    model: &Model,
    graph: &Graph,
    r#type: QueryPipelineType,
    mutation_mode: bool,
    json_value: &JsonValue
) -> Result<Vec<Document>, ActionError> {
    Ok(vec![doc!{}])
}