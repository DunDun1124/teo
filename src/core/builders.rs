use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::action::action::ActionType;
use crate::core::argument::{Argument, FnArgument};
use crate::core::argument::Argument::{PipelineArgument, ValueArgument};
use crate::core::connector::{ConnectorBuilder};
use crate::core::field::*;
use crate::core::pipeline::Pipeline;
use crate::core::value::Value;


pub struct GraphBuilder {
    pub(crate) enums: HashMap<&'static str, Vec<&'static str>>,
    pub(crate) models: Vec<ModelBuilder>,
    pub(crate) connector_builder: Option<Box<dyn ConnectorBuilder>>,
    pub(crate) reset_database: bool,
}

impl GraphBuilder {

    pub(crate) fn new() -> GraphBuilder {
        GraphBuilder {
            enums: HashMap::new(),
            models: Vec::new(),
            connector_builder: None,
            reset_database: false
        }
    }

    pub(crate) fn connector_builder(&self) -> &Box<dyn ConnectorBuilder> {
        match &self.connector_builder {
            Some(connector_builder) => connector_builder,
            None => panic!("Graph doesn't have a database connector.")
        }
    }

    pub fn r#enum(&mut self, name: &'static str, values: Vec<&'static str>) {
        self.enums.insert(name, values);
    }

    pub fn model<F: Fn(&mut ModelBuilder)>(&mut self, name: &'static str, build: F) {
        let mut model: ModelBuilder = ModelBuilder::new(name);
        build(&mut model);
        self.models.push(model);
    }

    pub fn reset_database(&mut self) {
        self.reset_database = true;
    }
}

pub struct ModelBuilder {
    pub name: &'static str,
    pub table_name: &'static str,
    pub url_segment_name: &'static str,
    pub localized_name: &'static str,
    pub description: &'static str,
    pub identity: bool,
    pub fields: Vec<FieldBuilder>,
    pub actions: HashSet<ActionType>
}

impl ModelBuilder {

    pub fn new(name: &'static str) -> ModelBuilder {
        return ModelBuilder {
            name,
            table_name: "",
            url_segment_name: "",
            localized_name: "",
            description: "",
            identity: false,
            fields: Vec::new(),
            actions: ActionType::default(),
        }
    }

    pub fn table_name(&mut self, table_name: &'static str) {
        self.table_name = table_name;
    }

    pub fn url_segment_name(&mut self, url_segment_name: &'static str) {
        self.url_segment_name = url_segment_name;
    }

    pub fn localized_name(&mut self, localized_name: &'static str) {
        self.localized_name = localized_name;
    }

    pub fn description(&mut self, description: &'static str) {
        self.description = description;
    }

    pub fn identity(&mut self) {
        self.identity = true;
    }

    pub fn field<F: Fn(&mut FieldBuilder)>(&mut self, name: &'static str, build: F) {
        let mut f = FieldBuilder::new(name);
        build(&mut f);
        self.fields.push(f);
    }

    pub fn internal(&mut self) {
        self.actions = HashSet::new();
    }

    pub fn enable<F: Fn(&mut ActionBuilder)>(&mut self, build: F) {
        self.internal();
        let mut action_builder = ActionBuilder::new();
        build(&mut action_builder);
        self.actions = action_builder.actions.clone();
    }

    pub fn disable<F: Fn(&mut ActionBuilder)>(&mut self, build: F) {
        let mut action_builder = ActionBuilder::new();
        build(&mut action_builder);
        self.actions = HashSet::from_iter(self.actions.difference(&action_builder.actions).map(|x| *x));
    }
}

pub struct ActionBuilder {
    pub(crate) actions: HashSet<ActionType>
}

impl ActionBuilder {
    pub(crate) fn new() -> Self {
        ActionBuilder { actions: HashSet::new() }
    }

    pub fn find_unique(&mut self) -> &mut Self {
        self.actions.insert(ActionType::FindUnique);
        self
    }

    pub fn find_first(&mut self) -> &mut Self {
        self.actions.insert(ActionType::FindFirst);
        self
    }

    pub fn find_many(&mut self) -> &mut Self {
        self.actions.insert(ActionType::FindMany);
        self
    }

    pub fn create(&mut self) -> &mut Self {
        self.actions.insert(ActionType::Create);
        self
    }

    pub fn update(&mut self) -> &mut Self {
        self.actions.insert(ActionType::Update);
        self
    }

    pub fn upsert(&mut self) -> &mut Self {
        self.actions.insert(ActionType::Upsert);
        self
    }

    pub fn delete(&mut self) -> &mut Self {
        self.actions.insert(ActionType::Delete);
        self
    }

    pub fn create_many(&mut self) -> &mut Self {
        self.actions.insert(ActionType::CreateMany);
        self
    }

    pub fn update_many(&mut self) -> &mut Self {
        self.actions.insert(ActionType::UpdateMany);
        self
    }

    pub fn delete_many(&mut self) -> &mut Self {
        self.actions.insert(ActionType::DeleteMany);
        self
    }

    pub fn count(&mut self) -> &mut Self {
        self.actions.insert(ActionType::Count);
        self
    }

    pub fn aggregate(&mut self) -> &mut Self {
        self.actions.insert(ActionType::Aggregate);
        self
    }

    pub fn group_by(&mut self) -> &mut Self {
        self.actions.insert(ActionType::GroupBy);
        self
    }
}

pub struct FieldBuilder {
    pub(crate) name: &'static str,
    pub(crate) localized_name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) r#type: Type,
    pub(crate) availability: Availability,
    pub(crate) store: Store,
    pub(crate) primary: bool,
    pub(crate) read_rule: ReadRule,
    pub(crate) write_rule: WriteRule,
    pub(crate) index: FieldIndex,
    pub(crate) query_ability: QueryAbility,
    pub(crate) object_assignment: ObjectAssignment,
    pub(crate) assigned_by_database: bool,
    pub(crate) auto_increment: bool,
    pub(crate) auth_identity: bool,
    pub(crate) default: Option<Argument>,
    pub(crate) on_set_pipeline: Pipeline,
    pub(crate) on_save_pipeline: Pipeline,
    pub(crate) on_output_pipeline: Pipeline,
}

impl FieldBuilder {
    pub fn new(name: &'static str) -> Self {
        return FieldBuilder {
            name,
            localized_name: "",
            description: "",
            r#type: Type::Undefined,
            availability: Availability::Required,
            store: Store::Embedded,
            primary: false,
            read_rule: ReadRule::Read,
            write_rule: WriteRule::Write,
            index: FieldIndex::NoIndex,
            query_ability: QueryAbility::Queryable,
            object_assignment: ObjectAssignment::Reference,
            assigned_by_database: false,
            auto_increment: false,
            auth_identity: false,
            default: None,
            on_set_pipeline: Pipeline::new(),
            on_save_pipeline: Pipeline::new(),
            on_output_pipeline: Pipeline::new(),
        }
    }

    pub fn localized_name(&mut self, localized_name: &'static str) {
        self.localized_name = localized_name;
    }

    pub fn description(&mut self, description: &'static str) {
        self.description = description;
    }

    pub fn object_id(&mut self) -> &mut Self {
        self.r#type = Type::ObjectId;
        return self;
    }

    pub fn bool(&mut self) -> &mut Self {
        self.r#type = Type::Bool;
        return self;
    }

    pub fn i8(&mut self) -> &mut Self {
        self.r#type = Type::I8;
        return self;
    }

    pub fn i16(&mut self) -> &mut Self {
        self.r#type = Type::I16;
        return self;
    }

    pub fn i32(&mut self) -> &mut Self {
        self.r#type = Type::I32;
        return self;
    }

    pub fn i64(&mut self) -> &mut Self {
        self.r#type = Type::I64;
        return self;
    }

    pub fn i128(&mut self) -> &mut Self {
        self.r#type = Type::I128;
        return self;
    }

    pub fn u8(&mut self) -> &mut Self {
        self.r#type = Type::U8;
        return self;
    }

    pub fn u16(&mut self) -> &mut Self {
        self.r#type = Type::U16;
        return self;
    }

    pub fn u32(&mut self) -> &mut Self {
        self.r#type = Type::U32;
        return self;
    }

    pub fn u64(&mut self) -> &mut Self {
        self.r#type = Type::U64;
        return self;
    }

    pub fn u128(&mut self) -> &mut Self {
        self.r#type = Type::U128;
        return self;
    }

    pub fn f32(&mut self) -> &mut Self {
        self.r#type = Type::F32;
        return self;
    }

    pub fn f64(&mut self) -> &mut Self {
        self.r#type = Type::F64;
        return self;
    }

    pub fn string(&mut self) -> &mut Self {
        self.r#type = Type::String;
        return self;
    }

    pub fn date(&mut self) -> &mut Self {
        self.r#type = Type::Date;
        return self;
    }

    pub fn datetime(&mut self) -> &mut Self {
        self.r#type = Type::DateTime;
        return self;
    }

    pub fn r#enum(&mut self, name: &'static str) -> &mut Self {
        self.r#type = Type::Enum(name);
        self
    }

    pub fn vec<F: Fn(&mut FieldBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = FieldBuilder::new("");
        build(&mut builder);
        let field = Field::new(&builder);
        self.r#type = Type::Vec(Box::new(field));
        return self;
    }

    pub fn map<F: Fn(&mut FieldBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = FieldBuilder::new("");
        build(&mut builder);
        let field = Field::new(&builder);
        self.r#type = Type::Map(Box::new(field));
        return self;
    }

    pub fn object(&mut self, model: &'static str) -> &mut Self {
        self.r#type = Type::Object(model);
        return self;
    }

    pub fn primary(&mut self) -> &mut Self {
        self.primary = true;
        return self;
    }

    pub fn internal(&mut self) -> &mut Self {
        self.write_rule = WriteRule::NoWrite;
        self.read_rule = ReadRule::NoRead;
        return self;
    }

    pub fn readonly(&mut self) -> &mut Self {
        self.write_rule = WriteRule::NoWrite;
        self
    }

    pub fn writeonly(&mut self) -> &mut Self {
        self.read_rule = ReadRule::NoRead;
        self.query_ability = QueryAbility::Unqueryable;
        self
    }

    pub fn write_once(&mut self) -> &mut Self {
        self.write_rule = WriteRule::WriteOnce;
        self
    }

    pub fn write_on_create(&mut self) -> &mut Self {
        self.write_rule = WriteRule::WriteOnCreate;
        self
    }

    pub fn write_nonnull(&mut self) -> &mut Self {
        self.write_rule = WriteRule::WriteNonNull;
        self
    }

    pub fn unique(&mut self) -> &mut Self {
        self.index = FieldIndex::Unique;
        return self;
    }

    pub fn compound_unique(&mut self, key: &'static str) -> &mut Self {
        self.index = FieldIndex::CompoundUnique(key);
        return self;
    }

    pub fn index(&mut self) -> &mut Self {
        self.index = FieldIndex::Index;
        return self;
    }

    pub fn compound_index(&mut self, key: &'static str) -> &mut Self {
        self.index = FieldIndex::CompoundIndex(key);
        return self;
    }

    pub fn optional(&mut self) -> &mut Self {
        self.availability = Availability::Optional;
        return self;
    }

    pub fn required(&mut self) -> &mut Self {
        self.availability = Availability::Required;
        self
    }

    pub fn linked_by(&mut self, field: &'static str) -> &mut Self {
        self.store = Store::ForeignKey(field);
        self
    }

    pub fn link_to(&mut self) -> &mut Self {
        self.store = Store::LocalKey;
        self
    }

    pub fn temp(&mut self) -> &mut Self {
        self.store = Store::Temp;
        self
    }

    pub fn calculated(&mut self) -> &mut Self {
        self.store = Store::Calculated;
        self.write_rule = WriteRule::NoWrite;
        self
    }

    pub fn copy(&mut self) -> &mut Self {
        self.object_assignment = ObjectAssignment::Copy;
        self
    }

    pub fn auth_identity(&mut self) -> &mut Self {
        self.auth_identity = true;
        self
    }

    pub fn assigned_by_database(&mut self) -> &mut Self {
        self.assigned_by_database = true;
        self
    }

    pub fn auto_increment(&mut self) -> &mut Self {
        self.assigned_by_database = true;
        self.auto_increment = true;
        self
    }

    pub fn on_set<F: Fn(&mut Pipeline)>(&mut self, build: F) -> &mut Self {
        build(&mut self.on_set_pipeline);
        self
    }

    pub fn on_save<F: Fn(&mut Pipeline)>(&mut self, build: F) -> &mut Self {
        build(&mut self.on_save_pipeline);
        return self;
    }

    pub fn on_output<F: Fn(&mut Pipeline)>(&mut self, build: F) -> &mut Self {
        build(&mut self.on_output_pipeline);
        return self;
    }

    pub fn assign_identity(&mut self) -> &mut Self {
        return self;
    }

    pub fn default(&mut self, value: Value) -> &mut Self {
        self.default = Some(ValueArgument(value));
        return self;
    }

    pub fn default_by_pipeline<F: Fn(&mut Pipeline)>(&mut self, build: F) -> &mut Self {
        let mut pipeline = Pipeline::new();
        build(&mut pipeline);
        self.default = Some(PipelineArgument(pipeline));
        return self;
    }

    pub fn default_by_fn(&mut self, function: Arc<dyn FnArgument>) -> &mut Self {
        self.default = Some(Argument::FunctionArgument(function));
        return self;
    }
}