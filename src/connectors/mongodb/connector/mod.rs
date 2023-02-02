pub mod builder;
pub mod save_session;

use std::collections::{HashMap};
use std::fmt::{Debug};
use std::ops::Neg;
use std::sync::Arc;
use std::sync::atomic::{Ordering};
use async_trait::async_trait;
use bson::{Bson, doc, Document};
use futures_util::StreamExt;
use key_path::path;
use mongodb::{options::ClientOptions, Client, Database, Collection, IndexModel};
use mongodb::error::{ErrorKind, WriteFailure, Error as MongoDBError};
use mongodb::options::{FindOneAndUpdateOptions, IndexOptions, ReturnDocument};
use regex::Regex;
use rust_decimal::Decimal;
use crate::connectors::mongodb::aggregation::Aggregation;
use crate::connectors::mongodb::bson::decoder::BsonDecoder;
use crate::connectors::mongodb::connector::save_session::MongoDBSaveSession;
use crate::core::connector::Connector;
use crate::core::env::Env;
use crate::core::env::intent::Intent;
use crate::core::object::Object;
use crate::core::field::Sort;
use crate::core::graph::Graph;
use crate::core::model::{Model};
use crate::core::model::index::{ModelIndex, ModelIndexType};
use crate::core::connector::SaveSession;
use crate::core::teon::Value;
use crate::core::error::ActionError;
use crate::core::input::Input;
use crate::core::result::ActionResult;
use crate::core::teon::decoder::Decoder;
use crate::teon;

#[derive(Debug)]
pub struct MongoDBConnector {
    client: Client,
    database: Database,
    collections: HashMap<String, Collection<Document>>
}

impl MongoDBConnector {
    pub(crate) async fn new(url: String, models: &Vec<Model>, reset_database: bool) -> MongoDBConnector {
        let options = match ClientOptions::parse(url).await {
            Ok(options) => options,
            Err(_) => panic!("MongoDB url is invalid.")
        };
        let database_name = match &options.default_database {
            Some(database_name) => database_name,
            None => panic!("No database name found in MongoDB url.")
        };
        let client = match Client::with_options(options.clone()) {
            Ok(client) => client,
            Err(_) => panic!("MongoDB client creating error.")
        };
        match client.database("xxxxxpingpingpingxxxxx").run_command(doc! {"ping": 1}, None).await {
            Ok(_) => (),
            Err(_) => panic!("Cannot connect to MongoDB database."),
        }
        let database = client.database(&database_name);
        if reset_database {
            let _ = database.drop(None).await;
        }
        let mut collections: HashMap<String, Collection<Document>> = HashMap::new();
        for model in models {
            let name = model.name();
            let collection: Collection<Document> = database.collection(model.table_name());
            let mut reviewed_names: Vec<String> = Vec::new();
            let cursor_result = collection.list_indexes(None).await;
            if cursor_result.is_ok() {
                let mut cursor = cursor_result.unwrap();
                while let Some(Ok(index)) = cursor.next().await {
                    if index.keys == doc!{"_id": 1} {
                        continue
                    }
                    let name = (&index).options.as_ref().unwrap().name.as_ref().unwrap();
                    let result = model.indices().iter().find(|i| i.name() == name);
                    if result.is_none() {
                        // not in our model definition, but in the database
                        // drop this index
                        let _ = collection.drop_index(name, None).await.unwrap();
                    } else {
                        let result = result.unwrap();
                        let our_format_index: ModelIndex = (&index).into();
                        if result != &our_format_index {
                            // alter this index
                            // drop first
                            let _ = collection.drop_index(name, None).await.unwrap();
                            // create index
                            let index_options = IndexOptions::builder()
                                .name(result.name().to_owned())
                                .unique(result.r#type() == ModelIndexType::Unique || result.r#type() == ModelIndexType::Primary)
                                .sparse(true)
                                .build();
                            let mut keys = doc!{};
                            for item in result.items() {
                                let field = model.field(item.field_name()).unwrap();
                                let column_name = field.column_name();
                                keys.insert(column_name, if item.sort() == Sort::Asc { 1 } else { -1 });
                            }
                            let index_model = IndexModel::builder().keys(keys).options(index_options).build();
                            let _result = collection.create_index(index_model, None).await;
                        }
                    }
                    reviewed_names.push(name.clone());
                }
            }
            for index in model.indices() {
                if !reviewed_names.contains(&index.name().to_string()) {
                    // create this index
                    let index_options = IndexOptions::builder()
                        .name(index.name().to_owned())
                        .unique(index.r#type() == ModelIndexType::Unique || index.r#type() == ModelIndexType::Primary)
                        .sparse(true)
                        .build();
                    let mut keys = doc!{};
                    for item in index.items() {
                        let field = model.field(item.field_name()).unwrap();
                        let column_name = field.column_name();
                        keys.insert(column_name, if item.sort() == Sort::Asc { 1 } else { -1 });
                    }
                    let index_model = IndexModel::builder().keys(keys).options(index_options).build();
                    let _result = collection.create_index(index_model, None).await;
                }
            }
            collections.insert(name.to_owned(), collection);
        }
        MongoDBConnector {
            client,
            database,
            collections
        }
    }

    fn document_to_object(&self, document: &Document, object: &Object, select: Option<&Value>, include: Option<&Value>) -> ActionResult<()> {
        for key in document.keys() {
            let object_field = object.model().fields().iter().find(|f| f.column_name() == key);
            if object_field.is_some() {
                // field
                let object_field = object_field.unwrap();
                let object_key = &object_field.name;
                let field_type = &object_field.field_type;
                let bson_value = document.get(key).unwrap();
                let value_result = BsonDecoder::decode(object.model(), object.graph(), field_type, object_field.is_optional(), bson_value, path![]);
                match value_result {
                    Ok(value) => {
                        object.inner.value_map.lock().unwrap().insert(object_key.to_string(), value);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            } else {
                // relation
                let relation = object.model().relation(key);
                if relation.is_none() {
                    continue;
                }
                let inner_finder = if let Some(include) = include {
                    include.get(key)
                } else {
                    None
                };
                let inner_select = if let Some(inner_finder) = inner_finder {
                    inner_finder.get("select")
                } else {
                    None
                };
                let inner_include = if let Some(inner_finder) = inner_finder {
                    inner_finder.get("include")
                } else {
                    None
                };
                let relation = relation.unwrap();
                let model_name = relation.model();
                let object_bsons = document.get(key).unwrap().as_array().unwrap();
                let mut related: Vec<Object> = vec![];
                for related_object_bson in object_bsons {
                    let env = object.env().alter_intent(Intent::NestedIncluded);
                    let related_object = object.graph().new_object(model_name, env)?;
                    self.document_to_object(related_object_bson.as_document().unwrap(), &related_object, inner_select, inner_include)?;
                    related.push(related_object);
                }
                object.inner.relation_query_map.lock().unwrap().insert(key.to_string(), related);
            }
        }
        object.inner.is_initialized.store(true, Ordering::SeqCst);
        object.inner.is_new.store(false, Ordering::SeqCst);
        object.set_select(select).unwrap();
        Ok(())
    }

    fn _handle_write_error(&self, error_kind: &ErrorKind) -> ActionError {
        return match error_kind {
            ErrorKind::Write(write) => {
                match write {
                    WriteFailure::WriteError(write_error) => {
                        match write_error.code {
                            11000 => {
                                let regex = Regex::new(r"dup key: \{ (.+?):").unwrap();
                                let field = regex.captures(write_error.message.as_str()).unwrap().get(1).unwrap().as_str();
                                ActionError::unique_value_duplicated(field, "''")
                            }
                            _ => {
                                ActionError::unknown_database_write_error()
                            }
                        }
                    }
                    _ => {
                        ActionError::unknown_database_write_error()
                    }
                }
            }
            _ => {
                ActionError::unknown_database_write_error()
            }
        }
    }

    async fn aggregate_or_group_by(&self, graph: &Graph, model: &Model, finder: &Value) -> Result<Vec<Value>, ActionError> {
        let aggregate_input = Aggregation::build_for_aggregate(model, graph, finder)?;
        let col = &self.collections[model.name()];
        let cur = col.aggregate(aggregate_input, None).await;
        if cur.is_err() {
            println!("{:?}", cur);
            return Err(ActionError::unknown_database_find_error());
        }
        let cur = cur.unwrap();
        let results: Vec<Result<Document, MongoDBError>> = cur.collect().await;
        let mut final_retval: Vec<Value> = vec![];
        for result in results.iter() {
            // there are records
            let data = result.as_ref().unwrap();
            let mut retval = teon!({});
            for (g, o) in data {
                if g.as_str() == "_id" {
                    continue;
                }
                // aggregate
                if g.starts_with("_") {
                    retval.as_hashmap_mut().unwrap().insert(g.clone(), teon!({}));
                    for (dbk, v) in o.as_document().unwrap() {
                        let k = dbk;
                        if let Some(f) = v.as_f64() {
                            retval.as_hashmap_mut().unwrap().get_mut(g.as_str()).unwrap().as_hashmap_mut().unwrap().insert(k.to_string(), teon!(f));
                        } else if let Some(i) = v.as_i64() {
                            retval.as_hashmap_mut().unwrap().get_mut(g.as_str()).unwrap().as_hashmap_mut().unwrap().insert(k.to_string(), teon!(i));
                        } else if let Some(i) = v.as_i32() {
                            retval.as_hashmap_mut().unwrap().get_mut(g.as_str()).unwrap().as_hashmap_mut().unwrap().insert(k.to_string(), teon!(i));
                        } else if v.as_null().is_some() {
                            retval.as_hashmap_mut().unwrap().get_mut(g.as_str()).unwrap().as_hashmap_mut().unwrap().insert(k.to_string(), teon!(null));
                        }
                    }
                } else {
                    // group by field
                    let field = model.field(g).unwrap();
                    let val = if o.as_null().is_some() { Value::Null } else {
                        BsonDecoder::decode(model, graph, field.r#type(), true, o, path![])?
                    };
                    let json_val = val;
                    retval.as_hashmap_mut().unwrap().insert(g.to_string(), json_val);
                }
            }
            final_retval.push(retval);
        }
        Ok(final_retval)
    }

    async fn create_object(&self, object: &Object) -> ActionResult<()> {
        let model = object.model();
        let keys = object.keys_for_save();
        let col = &self.collections[model.name()];
        let auto_keys = model.auto_keys();
        // create
        let mut doc = doc!{};
        for key in keys {
            if let Some(field) = model.field(key) {
                let column_name = field.column_name();
                let val: Bson = object.get_value(&key).unwrap().into();
                if val != Bson::Null {
                    doc.insert(column_name, val);
                }
            } else if let Some(property) = model.property(key) {
                let val: Value = object.get_property(key).await.unwrap();
                let val: Bson = val.into();
                if val != Bson::Null {
                    doc.insert(key, val);
                }
            }
        }
        let result = col.insert_one(doc, None).await;
        match result {
            Ok(insert_one_result) => {
                let id = insert_one_result.inserted_id.as_object_id().unwrap();
                for key in auto_keys {
                    let field = model.field(key).unwrap();
                    if field.column_name() == "_id" {
                        object.set_value(field.name(), Value::ObjectId(id))?;
                    }
                }
            }
            Err(error) => {
                return Err(self._handle_write_error(&error.kind));
            }
        }
        Ok(())
    }

    async fn update_object(&self, object: &Object) -> ActionResult<()> {
        let model = object.model();
        let keys = object.keys_for_save();
        let col = &self.collections[model.name()];
        let identifier: Bson = object.db_identifier().into();
        let identifier = identifier.as_document().unwrap();
        let mut set = doc!{};
        let mut unset = doc!{};
        let mut inc = doc!{};
        let mut mul = doc!{};
        let mut push = doc!{};
        for key in keys {
            if let Some(field) = model.field(key) {
                let column_name = field.column_name();
                if let Some(updator) = object.get_atomic_updator(key) {
                    let (key, val) = Input::key_value(updator.as_hashmap().unwrap());
                    match key {
                        "increment" => inc.insert(column_name, Bson::from(val)),
                        "decrement" => inc.insert(column_name, Bson::from(&val.neg())),
                        "multiply" => mul.insert(column_name, Bson::from(val)),
                        "divide" => mul.insert(column_name, Bson::Double(val.recip())),
                        "push" => push.insert(column_name, Bson::from(val)),
                        _ => panic!("Unhandled key."),
                    };
                } else {
                    let val = object.get_value(key).unwrap();
                    let bson_val: Bson = val.into();
                    if bson_val == Bson::Null {
                        unset.insert(key, bson_val);
                    } else {
                        set.insert(key, bson_val);
                    }
                }
            } else if let Some(_) = model.property(key) {
                let val: Value = object.get_property(key).await.unwrap();
                let bson_val: Bson = val.into();
                if bson_val != Bson::Null {
                    set.insert(key, bson_val);
                } else {
                    unset.insert(key, bson_val);
                }
            }
        }
        let mut update_doc = doc!{};
        let mut return_new = false;
        if !set.is_empty() {
            update_doc.insert("$set", set);
        }
        if !unset.is_empty() {
            update_doc.insert("$unset", unset);
        }
        if !inc.is_empty() {
            update_doc.insert("$inc", inc);
            return_new = true;
        }
        if !mul.is_empty() {
            update_doc.insert("$mul", mul);
            return_new = true;
        }
        if !push.is_empty() {
            update_doc.insert("$push", push);
            return_new = true;
        }
        if update_doc.is_empty() {
            return Ok(());
        }
        if !return_new {
            let result = col.update_one(identifier.clone(), update_doc, None).await;
            return match result {
                Ok(_) => Ok(()),
                Err(error) => {
                    Err(self._handle_write_error(&error.kind))
                }
            }
        } else {
            let options = FindOneAndUpdateOptions::builder().return_document(ReturnDocument::After).build();
            let result = col.find_one_and_update(identifier.clone(), update_doc, options).await;
            match result {
                Ok(updated_document) => {
                    for key in object.inner.atomic_updator_map.lock().unwrap().keys() {
                        let bson_new_val = updated_document.as_ref().unwrap().get(key).unwrap();
                        let field = object.model().field(key).unwrap();
                        let field_value = BsonDecoder::decode(model, object.graph(), field.r#type(), field.is_optional(), bson_new_val, path![])?;
                        object.inner.value_map.lock().unwrap().insert(key.to_string(), field_value);
                    }
                }
                Err(error) => {
                    return Err(self._handle_write_error(&error.kind));
                }
            }
        }
        Ok(())
    }

}

#[async_trait]
impl Connector for MongoDBConnector {

    async fn save_object(&self, object: &Object, session: Arc<dyn SaveSession>) -> ActionResult<()> {
        if object.inner.is_new.load(Ordering::SeqCst) {
            self.create_object(object).await
        } else {
            self.update_object(object).await
        }
    }

    async fn delete_object(&self, object: &Object, session: Arc<dyn SaveSession>) -> ActionResult<()> {
        if object.inner.is_new.load(Ordering::SeqCst) {
            return Err(ActionError::object_is_not_saved());
        }
        let model = object.model();
        let col = &self.collections[model.name()];
        let bson_identifier: Bson = object.db_identifier().into();
        let document_identifier = bson_identifier.as_document().unwrap();
        let result = col.delete_one(document_identifier.clone(), None).await;
        return match result {
            Ok(_result) => Ok(()),
            Err(_err) => {
                Err(ActionError::unknown_database_delete_error())
            }
        }
    }

    async fn find_unique(&self, graph: &Graph, model: &Model, finder: &Value, mutation_mode: bool, env: Env) -> Result<Object, ActionError> {
        let select = finder.get("select");
        let include = finder.get("include");

        let aggregate_input = Aggregation::build(model, graph, finder)?;
        let col = &self.collections[model.name()];
        let cur = col.aggregate(aggregate_input, None).await;
        if cur.is_err() {
            return Err(ActionError::unknown_database_find_unique_error());
        }
        let cur = cur.unwrap();
        let results: Vec<Result<Document, MongoDBError>> = cur.collect().await;
        if results.is_empty() {
            return Err(ActionError::object_not_found());
        }
        for doc in results {
            let obj = graph.new_object(model.name(), env)?;
            self.document_to_object(&doc.unwrap(), &obj, select, include)?;
            return Ok(obj);
        }
        Err(ActionError::object_not_found())
    }

    async fn find_many(&self, graph: &Graph, model: &Model, finder: &Value, mutation_mode: bool, env: Env) -> Result<Vec<Object>, ActionError> {
        let select = finder.get("select");
        let include = finder.get("include");
        let aggregate_input = Aggregation::build(model, graph, finder)?;
        let reverse = Input::has_negative_take(finder);
        let col = &self.collections[model.name()];
        // println!("see aggregate input: {:?}", aggregate_input);
        let cur = col.aggregate(aggregate_input, None).await;
        if cur.is_err() {
            println!("{:?}", cur);
            return Err(ActionError::unknown_database_find_error());
        }
        let cur = cur.unwrap();
        let mut result: Vec<Object> = vec![];
        let results: Vec<Result<Document, MongoDBError>> = cur.collect().await;
        for doc in results {
            let obj = graph.new_object(model.name(), env.clone())?;
            match self.document_to_object(&doc.unwrap(), &obj, select, include) {
                Ok(_) => {
                    if reverse {
                        result.insert(0, obj);
                    } else {
                        result.push(obj);
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(result)
    }

    async fn count(&self, graph: &Graph, model: &Model, finder: &Value) -> Result<usize, ActionError> {
        let input = Aggregation::build_for_count(model, graph, finder)?;
        let col = &self.collections[model.name()];
        let cur = col.aggregate(input, None).await;
        if cur.is_err() {
            println!("{:?}", cur);
            return Err(ActionError::unknown_database_find_error());
        }
        let cur = cur.unwrap();
        let results: Vec<Result<Document, MongoDBError>> = cur.collect().await;
        if results.is_empty() {
            Ok(0)
        } else {
            let v = results.get(0).unwrap().as_ref().unwrap();
            let bson_count = v.get("count").unwrap();
            match bson_count {
                Bson::Int32(i) => Ok(*i as usize),
                Bson::Int64(i) => Ok(*i as usize),
                _ => panic!("Unhandled count number type.")
            }
        }
    }

    async fn aggregate(&self, graph: &Graph, model: &Model, finder: &Value) -> Result<Value, ActionError> {
        let results = self.aggregate_or_group_by(graph, model, finder).await?;
        if results.is_empty() {
            // there is no record
            let mut retval = teon!({});
            for (g, o) in finder.as_hashmap().unwrap() {
                retval.as_hashmap_mut().unwrap().insert(g.clone(), teon!({}));
                for (k, _v) in o.as_hashmap().unwrap() {
                    let value = if g == "_count" { teon!(0) } else { teon!(null) };
                    retval.as_hashmap_mut().unwrap().get_mut(g.as_str()).unwrap().as_hashmap_mut().unwrap().insert(k.to_string(), value);
                }
            }
            Ok(retval)
        } else {
            Ok(results.get(0).unwrap().clone())
        }
    }

    async fn group_by(&self, graph: &Graph, model: &Model, finder: &Value) -> Result<Value, ActionError> {
        Ok(Value::Vec(self.aggregate_or_group_by(graph, model, finder).await?))
    }

    fn new_save_session(&self) -> Arc<dyn SaveSession> {
        Arc::new(MongoDBSaveSession {})
    }
}

unsafe impl Sync for MongoDBConnector {}
unsafe impl Send for MongoDBConnector {}
