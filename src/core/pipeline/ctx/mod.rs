pub mod validity;

use key_path::KeyPath;
use crate::core::object::Object;
use crate::core::result::Result;
use crate::core::teon::Value;
use crate::prelude::Error;

#[derive(Clone)]
pub struct Ctx<'a> {
    pub(crate) value: Value,
    pub(crate) object: Option<Object>,
    pub(crate) path: KeyPath<'a>,
}

impl<'a> Ctx<'a> {

    pub(crate) fn initial_state_with_value(value: Value) -> Self {
        Self {
            value,
            object: None,
            path: KeyPath::default(),
        }
    }

    pub(crate) fn initial_state_with_object(object: Object) -> Self {
        Self {
            value: Value::Null,
            object: Some(object),
            path: KeyPath::default(),
        }
    }

    pub(crate) fn with_path(&self, path: impl AsRef<KeyPath<'a>>) -> Self {
        Self {
            value: self.value.clone(),
            object: self.object.clone(),
            path: path.as_ref().clone(),
        }
    }

    pub(crate) fn with_value(&self, value: Value) -> Self {
        Self {
            value,
            object: self.object.clone(),
            path: self.path.clone(),
        }
    }

    pub(crate) fn with_value_result(&self, result: Result<Value>) -> Result<Self> {
        match result {
            Ok(value) => Ok(Self {
                value,
                object: self.object.clone(),
                path: self.path.clone(),
            }),
            Err(err) => Err(err),
        }
    }

    pub(crate) fn with_invalid(&self, reason: impl Into<String>) -> Error {
        Error::validation_error(&self.path, reason.into())
    }

    pub(crate) fn get_value(&self) -> Value {
        self.value.clone()
    }

    pub(crate) fn get_object(&self) -> Result<Object> {
        match &self.object {
            Some(object) => Ok(object.clone()),
            None => Err(Error::internal_server_error_with_path(&self.path, "ctx object is null"))
        }
    }

    pub(crate) fn internal_server_error(&self, reason: impl Into<String>) -> Error {
        Error::internal_server_error_with_path(&self.path, reason.into())
    }
}