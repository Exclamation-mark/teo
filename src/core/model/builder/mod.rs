use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::Arc;
use inflector::Inflector;
use crate::core::action::builder::ActionsBuilder;
use crate::core::action::r#type::ActionType;
use crate::core::connector::{ConnectorBuilder};
use crate::core::field::*;
use crate::core::field::builder::FieldBuilder;
use crate::core::permission::builder::PermissionBuilder;
use crate::core::relation::builder::RelationBuilder;
use crate::core::field::ReadRule::NoRead;
use crate::core::field::Store::{Calculated, Temp};
use crate::core::field::WriteRule::NoWrite;
use crate::core::model::{ModelIndex, ModelIndexItem, Model, ModelIndexType, ModelInner};
use crate::core::model::builder::index_builder::ModelIndexBuilder;
use crate::core::model_callback::PinFutureObj;
use crate::core::relation::Relation;
use crate::core::object::Object;
use crate::core::error::ActionError;

pub(crate) mod index_builder;

pub struct ModelBuilder {
    pub(crate) name: String,
    pub(crate) table_name: String,
    pub(crate) url_segment_name: String,
    pub(crate) localized_name: String,
    pub(crate) description: String,
    pub(crate) identity: bool,
    pub(crate) field_builders: Vec<FieldBuilder>,
    pub(crate) relation_builders: Vec<RelationBuilder>,
    pub(crate) actions: HashSet<ActionType>,
    pub(crate) permission: Option<PermissionBuilder>,
    pub(crate) primary: Option<ModelIndex>,
    pub(crate) indices: Vec<ModelIndex>,
    pub(crate) after_save_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) after_update_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) after_create_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) after_delete_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) before_save_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) before_update_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) before_create_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) before_delete_fns: Vec<Arc<dyn Fn(Object) -> PinFutureObj<Result<(), ActionError>>>>,
    pub(crate) actions_builder: ActionsBuilder,
    connector_builder: * const Box<dyn ConnectorBuilder>,
}

impl ModelBuilder {

    pub(crate) fn new(name: impl Into<String>, connector_builder: &Box<dyn ConnectorBuilder>) -> Self {
        Self {
            name: name.into(),
            table_name: "".to_string(),
            url_segment_name: "".to_string(),
            localized_name: "".to_string(),
            description: "".to_string(),
            identity: false,
            field_builders: Vec::new(),
            relation_builders: Vec::new(),
            actions: ActionType::default(),
            permission: None,
            primary: None,
            indices: Vec::new(),
            after_create_fns: Vec::new(),
            after_update_fns: Vec::new(),
            after_save_fns: Vec::new(),
            after_delete_fns: Vec::new(),
            before_create_fns: Vec::new(),
            before_update_fns: Vec::new(),
            before_save_fns: Vec::new(),
            before_delete_fns: Vec::new(),
            actions_builder: ActionsBuilder::new(),
            connector_builder
        }
    }

    fn connector_builder(&self) -> &Box<dyn ConnectorBuilder> {
        unsafe {
            &*self.connector_builder
        }
    }

    pub fn table_name(&mut self, table_name: impl Into<String>) -> &mut Self {
        self.table_name = table_name.into();
        self
    }

    pub fn url_segment_name(&mut self, url_segment_name: impl Into<String>) -> &mut Self {
        self.url_segment_name = url_segment_name.into();
        self
    }

    pub fn localized_name(&mut self, localized_name: impl Into<String>) -> &mut Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = description.into();
        self
    }

    pub fn identity(&mut self) -> &mut Self {
        self.identity = true;
        self.actions.insert(ActionType::SignIn);
        self.actions.insert(ActionType::Identity);
        self
    }

    pub fn field<F: Fn(&mut FieldBuilder)>(&mut self, name: &'static str, build: F) -> &mut Self {
        let mut f = FieldBuilder::new(name, self.connector_builder());
        build(&mut f);
        self.field_builders.push(f);
        self
    }

    pub fn relation<F: Fn(&mut RelationBuilder)>(&mut self, name: &'static str, build: F) -> &mut Self {
        let mut f = RelationBuilder::new(name, self.connector_builder());
        build(&mut f);
        self.relation_builders.push(f);
        self
    }

    pub fn internal(&mut self) -> &mut Self {
        self.actions = HashSet::new();
        self
    }

    pub fn permissions<F: Fn(&mut PermissionBuilder)>(&mut self, build: F) -> &mut Self {
        let mut permission_builder = PermissionBuilder::new();
        build(&mut permission_builder);
        self.permission = Some(permission_builder);
        self
    }

    pub fn primary<I, T>(&mut self, keys: I) -> &mut Self where I: IntoIterator<Item = T>, T: Into<String> {
        let string_keys: Vec<String> = keys.into_iter().map(Into::into).collect();
        let name = string_keys.join("_");
        let items = string_keys.iter().map(|k| {
            ModelIndexItem { field_name: k.to_string(), sort: Sort::Asc, len: None }
        }).collect();
        let index = ModelIndex {
            index_type: ModelIndexType::Primary,
            name,
            items
        };
        let primary_index = index.clone();
        self.indices.push(index);
        self.primary = Some(primary_index);
        self
    }

    pub fn primary_settings<F: Fn(&mut ModelIndexBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = ModelIndexBuilder::new(ModelIndexType::Primary);
        build(&mut builder);
        self.primary = Some(builder.build());
        self
    }

    pub fn index<I, T>(&mut self, keys: I) -> &mut Self where I: IntoIterator<Item = T>, T: Into<String> {
        let string_keys: Vec<String> = keys.into_iter().map(Into::into).collect();
        let name = string_keys.join("_");
        let items = string_keys.iter().map(|k| {
            ModelIndexItem { field_name: k.to_string(), sort: Sort::Asc, len: None }
        }).collect();
        let index = ModelIndex {
            index_type: ModelIndexType::Index,
            name,
            items
        };
        self.indices.push(index);
        self
    }

    pub fn index_settings<F: Fn(&mut ModelIndexBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = ModelIndexBuilder::new(ModelIndexType::Index);
        build(&mut builder);
        self.indices.push(builder.build());
        self
    }

    pub fn unique<I, T>(&mut self, keys: I) -> &mut Self where I: IntoIterator<Item = T>, T: Into<String> {
        let string_keys: Vec<String> = keys.into_iter().map(Into::into).collect();
        let name = string_keys.join("_");
        let items = string_keys.iter().map(|k| {
            ModelIndexItem { field_name: k.to_string(), sort: Sort::Asc, len: None }
        }).collect();
        let index = ModelIndex {
            index_type: ModelIndexType::Unique,
            name,
            items
        };
        self.indices.push(index);
        self
    }

    pub fn unique_settings<F: Fn(&mut ModelIndexBuilder)>(&mut self, build: F) -> &mut Self {
        let mut builder = ModelIndexBuilder::new(ModelIndexType::Unique);
        build(&mut builder);
        self.indices.push(builder.build());
        self
    }

    pub fn after_save<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.after_save_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn after_create<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.after_create_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn after_update<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.after_update_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn after_delete<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.after_delete_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn before_save<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.before_save_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn before_create<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.before_create_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn before_update<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.before_update_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn before_delete<F, Fut>(&mut self, callback: &'static F) -> &mut Self where F: (Fn(Object) -> Fut) + 'static, Fut: Future<Output = Result<(), ActionError>> + 'static {
        self.before_delete_fns.push(Arc::new(|object| Box::pin(callback(object))));
        self
    }

    pub fn actions<F: Fn(&mut ActionsBuilder)>(&mut self, build: F) -> &mut Self {
        build(&mut self.actions_builder);
        self
    }

    pub(crate) fn build(&self, connector_builder: &Box<dyn ConnectorBuilder>) -> Model {
        let all_keys = Self::all_keys(self);
        let input_keys = Self::allowed_input_keys(self);
        let save_keys = Self::allowed_save_keys(self);
        let output_keys = Self::allowed_output_keys(self);
        let get_value_keys = Self::get_get_value_keys(self);
        let query_keys = Self::get_query_keys(self);
        let auth_identity_keys = Self::get_auth_identity_keys(self);
        let auth_by_keys = Self::get_auth_by_keys(self);
        let fields_vec: Vec<Arc<Field>> = self.field_builders.iter().map(|fb| { Arc::new(fb.build(connector_builder)) }).collect();
        let relations_vec: Vec<Arc<Relation>> = self.relation_builders.iter().map(|rb| { Arc::new(rb.build(connector_builder)) }).collect();
        let mut fields_map: HashMap<String, Arc<Field>> = HashMap::new();
        let mut relations_map: HashMap<String, Arc<Relation>> = HashMap::new();
        let mut primary_field: Option<Arc<Field>> = None;
        let mut primary = self.primary.clone();
        let mut indices = self.indices.clone();
        for relation in relations_vec.iter() {
            relations_map.insert(relation.name.clone(), relation.clone());
        }
        for field in fields_vec.iter() {
            fields_map.insert(field.name.clone(), field.clone());
            if field.primary {
                primary_field = Some(field.clone());
                primary = Some(ModelIndex {
                    index_type: ModelIndexType::Primary,
                    name: "".to_string(),
                    items: vec![
                        ModelIndexItem {
                            field_name: field.name.clone(),
                            sort: Sort::Asc,
                            len: None
                        }
                    ]
                });
            }
            if field.index != FieldIndex::NoIndex {
                match &field.index {
                    FieldIndex::Index(settings) => {
                        indices.push(ModelIndex {
                            index_type: ModelIndexType::Index,
                            name: if settings.name.is_some() { settings.name.as_ref().unwrap().clone() } else { field.name.clone() },
                            items: vec![
                                ModelIndexItem {
                                    field_name: field.name.clone(),
                                    sort: settings.sort,
                                    len: settings.length
                                }
                            ]
                        })
                    }
                    FieldIndex::Unique(settings) => {
                        indices.push(ModelIndex {
                            index_type: ModelIndexType::Unique,
                            name: if settings.name.is_some() { settings.name.as_ref().unwrap().clone() } else { field.name.clone() },
                            items: vec![
                                ModelIndexItem {
                                    field_name: field.name.clone(),
                                    sort: settings.sort,
                                    len: settings.length
                                }
                            ]
                        })
                    }
                    _ => { }
                }
            }
        }

        if primary.is_none() {
            panic!("Model '{}' must has a primary field.", self.name);
        }

        let unique_query_keys = Self::get_unique_query_keys(self, &indices, primary.as_ref().unwrap());
        let _a = if self.url_segment_name == "" { self.name.to_kebab_case().to_plural() } else { self.url_segment_name.to_string() };
        let inner = ModelInner {
            name: self.name.clone(),
            table_name: if self.table_name == "" { self.name.to_lowercase().to_plural() } else { self.table_name.to_string() },
            url_segment_name: if self.url_segment_name == "" { self.name.to_kebab_case().to_plural() } else { self.url_segment_name.to_string() },
            localized_name: self.localized_name.clone(),
            description: self.description.clone(),
            identity: self.identity,
            actions: self.actions.clone(),
            permission: if let Some(builder) = &self.permission { Some(builder.build()) } else { None },
            fields_vec,
            fields_map,
            relations_map,
            relations_vec,
            primary: primary.unwrap(),
            indices: indices.clone(),
            after_save_fns: self.after_save_fns.clone(),
            after_create_fns: self.after_create_fns.clone(),
            after_update_fns: self.after_update_fns.clone(),
            after_delete_fns: self.after_delete_fns.clone(),
            before_save_fns: self.before_save_fns.clone(),
            before_create_fns: self.before_create_fns.clone(),
            before_update_fns: self.before_update_fns.clone(),
            before_delete_fns: self.before_delete_fns.clone(),
            primary_field,
            all_keys,
            input_keys,
            save_keys,
            output_keys,
            get_value_keys,
            query_keys,
            unique_query_keys,
            auth_identity_keys,
            auth_by_keys
        };
        Model::new_with_inner(Arc::new(inner))
    }

    fn all_relation_keys(&self) -> Vec<String> {
        self.relation_builders.iter().map(|r| r.name.clone()).collect()
    }

    fn all_keys(&self) -> Vec<String> {
        let mut fields: Vec<String> = self.field_builders.iter().map(|f| f.name.clone()).collect();
        fields.extend(self.all_relation_keys());
        fields
    }

    fn allowed_input_keys(&self) -> Vec<String> {
        let mut fields: Vec<String> = self.field_builders.iter()
            .filter(|&f| { f.write_rule != NoWrite })
            .map(|f| { f.name.clone() })
            .collect();
        fields.extend(self.all_relation_keys());
        fields
    }

    fn allowed_save_keys(&self) -> Vec<String> {
        let mut fields: Vec<String> = self.field_builders.iter()
            .filter(|&f| { f.store != Calculated && f.store != Temp })
            .map(|f| { f.name.clone() })
            .collect();
        fields.extend(self.all_relation_keys());
        fields
    }

    fn allowed_output_keys(&self) -> Vec<String> {
        let mut fields: Vec<String> = self.field_builders.iter()
            .filter(|&f| { f.read_rule != NoRead })
            .map(|f| { f.name.clone() })
            .collect();
        fields.extend(self.all_relation_keys());
        fields
    }

    pub(crate) fn get_get_value_keys(&self) -> Vec<String> {
        let mut fields: Vec<String> = self.field_builders.iter()
            .map(|f| { f.name.clone() })
            .collect();
        fields.extend(self.all_relation_keys());
        fields
    }

    pub(crate) fn get_query_keys(&self) -> Vec<String> {
        let mut fields: Vec<String> = self.field_builders.iter()
            .filter(|&f| { f.query_ability == QueryAbility::Queryable })
            .map(|f| { f.name.clone() })
            .collect();
        fields.extend(self.all_relation_keys());
        fields
    }

    pub(crate) fn get_unique_query_keys(&self, indices: &Vec<ModelIndex>, primary: &ModelIndex) -> Vec<HashSet<String>> {
        let mut result: Vec<HashSet<String>> = Vec::new();
        for index in indices {
            let set = HashSet::from_iter(index.items.iter().map(|i| {
                i.field_name.clone()
            }));
            result.push(set);
        }
        result.push(HashSet::from_iter(primary.items.iter().map(|i| i.field_name.clone())));
        result
    }

    pub(crate) fn get_auth_identity_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.auth_identity == true })
            .map(|f| { f.name.clone() })
            .collect()
    }

    pub(crate) fn get_auth_by_keys(&self) -> Vec<String> {
        self.field_builders.iter()
            .filter(|&f| { f.auth_by == true })
            .map(|f| { f.name.clone() })
            .collect()
    }
}