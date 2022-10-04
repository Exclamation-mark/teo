use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use serde::{Serialize, Serializer};
use maplit::hashmap;
use key_path::KeyPath;
use crate::core::model::Model;

#[derive(Debug, PartialEq, Serialize)]
pub enum ActionErrorType {
    UnknownDatabaseWriteError,
    UnknownDatabaseDeleteError,
    UnknownDatabaseFindError,
    UnknownDatabaseFindUniqueError,
    UnknownDatabaseCountError,
    NotFound,
    InternalServerError,
    UndefinedAction,
    UnallowedAction,
    ObjectNotFound,
    ObjectIsNotSaved,
    FieldIsNotUnique,
    UnmatchedDataTypeInDatabase,
    InvalidAuthorizationFormat,
    IdentityIsNotFound,
    UnexpectedNull,
    InvalidQueryInput,
    RequiredRelationCannotDisconnect,
    NewObjectCannotDisconnect,
    SaveCallingError,
    CustomError,
    ModelNotFound,
    WrongIdentityModel,
    PropertySetterError,

    // new errors

    // request format
    IncorrectJSONFormat,
    UnexpectedInputRootType,
    UnexpectedInputType,
    UnexpectedInputKey,
    UnexpectedInputValue,
    MissingRequiredInput,
    UnexpectedObjectLength,

    // request token
    InvalidJWTToken,

    // permission
    PermissionDenied,

    // object api
    InvalidKey,
}

impl ActionErrorType {
    pub fn code(&self) -> u16 {
        match self {
            ActionErrorType::IncorrectJSONFormat => { 400 }
            ActionErrorType::UndefinedAction => { 400 }
            ActionErrorType::UnallowedAction => { 400 }
            ActionErrorType::ObjectIsNotSaved => { 400 }
            ActionErrorType::UnknownDatabaseWriteError => { 500 }
            ActionErrorType::UnknownDatabaseDeleteError => { 500 }
            ActionErrorType::UnknownDatabaseFindError => { 500 }
            ActionErrorType::UnknownDatabaseFindUniqueError => { 500 }
            ActionErrorType::UnknownDatabaseCountError => { 500 }
            ActionErrorType::UnmatchedDataTypeInDatabase => { 500 }
            ActionErrorType::NotFound => { 404 }
            ActionErrorType::InternalServerError => { 500 }
            ActionErrorType::ObjectNotFound => { 404 }
            ActionErrorType::FieldIsNotUnique => { 400 }
            ActionErrorType::InvalidAuthorizationFormat => { 401 }
            ActionErrorType::InvalidJWTToken => { 401 }
            ActionErrorType::IdentityIsNotFound => { 401 }
            ActionErrorType::UnexpectedNull => { 400 }
            ActionErrorType::InvalidQueryInput => { 400 }
            ActionErrorType::RequiredRelationCannotDisconnect => { 400 }
            ActionErrorType::NewObjectCannotDisconnect => { 400 }
            ActionErrorType::SaveCallingError => { 500 }
            ActionErrorType::CustomError => { 500 }
            ActionErrorType::ModelNotFound => { 500 }
            ActionErrorType::WrongIdentityModel => { 401 }
            ActionErrorType::PropertySetterError => { 400 }
            ActionErrorType::UnexpectedInputRootType => { 400 }
            ActionErrorType::UnexpectedInputType => { 400 }
            ActionErrorType::UnexpectedInputKey => { 400 }
            ActionErrorType::UnexpectedInputValue => { 400 }
            ActionErrorType::MissingRequiredInput => { 400 }
            ActionErrorType::UnexpectedObjectLength => { 400 }
            ActionErrorType::InvalidKey => { 500 }
            ActionErrorType::PermissionDenied => { 401 }
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ActionError {
    pub r#type: ActionErrorType,
    pub message: String,
    pub errors: Option<HashMap<String, String>>
}

impl ActionError {

    pub fn invalid_query_input(reason: impl Into<String>) -> Self {
        ActionError {
            r#type: ActionErrorType::InvalidQueryInput,
            message: reason.into(),
            errors: None
        }
    }

    pub fn unexpected_enum_value(field: impl Into<String>) -> Self {
        let mut errors: HashMap<String, String> = HashMap::with_capacity(1);
        errors.insert(field.into(), "Enum value is unexpected.".to_string());
        ActionError {
            r#type: ActionErrorType::UnexpectedInputValue,
            message: "Enum value is unexpected.".to_string(),
            errors: Some(errors)
        }
    }

    pub fn unique_value_duplicated(field: impl Into<String>) -> Self {
        let mut errors: HashMap<String, String> = HashMap::with_capacity(1);
        errors.insert(field.into(), "Unique value duplicated.".to_string());
        ActionError {
            r#type: ActionErrorType::UnexpectedInputValue,
            message: "Input is not valid.".to_string(),
            errors: Some(errors)
        }
    }

    pub fn internal_server_error(reason: String) -> Self {
        ActionError {
            r#type: ActionErrorType::InternalServerError,
            message: reason,
            errors: None
        }
    }

    pub fn unknown_database_write_error() -> Self {
        ActionError {
            r#type: ActionErrorType::UnknownDatabaseWriteError,
            message: "An unknown database write error occurred.".to_string(),
            errors: None
        }
    }

    pub fn unknown_database_delete_error() -> Self {
        ActionError {
            r#type: ActionErrorType::UnknownDatabaseDeleteError,
            message: "An unknown database delete error occurred.".to_string(),
            errors: None
        }
    }

    pub fn not_found() -> Self {
        ActionError {
            r#type: ActionErrorType::NotFound,
            message: "The request destination is not found.".to_string(),
            errors: None
        }
    }

    pub fn undefined_action() -> Self {
        ActionError {
            r#type: ActionErrorType::UndefinedAction,
            message: "Undefined action.".to_string(),
            errors: None
        }
    }

    pub fn unallowed_action() -> Self {
        ActionError {
            r#type: ActionErrorType::UnallowedAction,
            message: "Unallowed action.".to_string(),
            errors: None
        }
    }

    pub fn object_not_found() -> Self {
        ActionError {
            r#type: ActionErrorType::ObjectNotFound,
            message: "The requested object is not exist.".to_string(),
            errors: None
        }
    }

    pub fn object_is_not_saved() -> Self {
        ActionError {
            r#type: ActionErrorType::ObjectIsNotSaved,
            message: "This object is not saved thus can't be deleted.".to_string(),
            errors: None
        }
    }

    pub fn field_is_not_unique() -> Self {
        ActionError {
            r#type: ActionErrorType::FieldIsNotUnique,
            message: format!("Unique where input is not unique."),
            errors: None
        }
    }

    pub fn unknown_database_find_error() -> Self {
        ActionError {
            r#type: ActionErrorType::UnknownDatabaseFindError,
            message: "An unknown query error occurred.".to_string(),
            errors: None
        }
    }

    pub fn unknown_database_find_unique_error() -> Self {
        ActionError {
            r#type: ActionErrorType::UnknownDatabaseFindUniqueError,
            message: "An unknown query unique error occurred.".to_string(),
            errors: None
        }
    }

    pub fn unknown_database_count_error() -> Self {
        ActionError {
            r#type: ActionErrorType::UnknownDatabaseCountError,
            message: "An unknown count error occurred.".to_string(),
            errors: None
        }
    }

    pub fn unmatched_data_type_in_database(field_name: &str) -> Self {
        ActionError {
            r#type: ActionErrorType::UnmatchedDataTypeInDatabase,
            message: format!("Unmatched data type for field '{field_name}' in database."),
            errors: None
        }
    }

    pub fn invalid_authorization_format() -> Self {
        ActionError {
            r#type: ActionErrorType::InvalidAuthorizationFormat,
            message: "Invalid authorization header format.".to_string(),
            errors: None
        }
    }

    pub fn invalid_jwt_token() -> Self {
        ActionError {
            r#type: ActionErrorType::InvalidJWTToken,
            message: "This token is malformed.".to_string(),
            errors: None
        }
    }

    pub fn identity_is_not_found() -> Self {
        ActionError {
            r#type: ActionErrorType::IdentityIsNotFound,
            message: "Identity is not found.".to_string(),
            errors: None
        }
    }

    pub fn unexpected_null(field: impl Into<String>) -> Self {
        let mut errors: HashMap<String, String> = HashMap::with_capacity(1);
        errors.insert(field.into(), "Unexpected null.".to_string());
        ActionError {
            r#type: ActionErrorType::UnexpectedNull,
            message: "Unexpected null.".to_string(),
            errors: Some(errors)
        }
    }

    pub fn expected(expected_json_type: impl Into<String>, field: impl Into<String>) -> Self {
        let expected_json_type = expected_json_type.into();
        let mut errors: HashMap<String, String> = HashMap::with_capacity(1);
        errors.insert(field.into(), "Unexpected field type.".to_string());
        ActionError {
            r#type: ActionErrorType::UnexpectedFieldType,
            message: format!("Expected '{expected_json_type}'."),
            errors: Some(errors)
        }
    }

    pub fn required_relation_cannot_disconnect() -> Self {
        ActionError {
            r#type: ActionErrorType::RequiredRelationCannotDisconnect,
            message: "Required relation cannot disconnect.".to_string(),
            errors: None
        }
    }

    pub fn new_object_cannot_disconnect() -> Self {
        ActionError {
            r#type: ActionErrorType::NewObjectCannotDisconnect,
            message: "New object cannot disconnect.".to_string(),
            errors: None
        }
    }

    pub fn save_calling_error(model_name: impl AsRef<str> + Display) -> Self {
        ActionError {
            r#type: ActionErrorType::SaveCallingError,
            message: format!("Model `{model_name}', save method cannot be called inside before save callbacks."),
            errors: None
        }
    }

    pub fn custom_error(message: impl Into<String>) -> Self {
        ActionError {
            r#type: ActionErrorType::CustomError,
            message: message.into(),
            errors: None
        }
    }

    pub fn model_not_found(name: impl AsRef<str> + Display) -> Self {
        ActionError {
            r#type: ActionErrorType::ModelNotFound,
            message: format!("Model named `{name}' is not found."),
            errors: None
        }
    }

    pub fn wrong_identity_model() -> Self {
        ActionError {
            r#type: ActionErrorType::WrongIdentityModel,
            message: format!("This identity is valid but is not of this model."),
            errors: None
        }
    }

    pub fn property_setter_error(reason: impl Into<String>) -> Self {
        ActionError {
            r#type: ActionErrorType::PropertySetterError,
            message: reason.into(),
            errors: None
        }
    }

    // new error types which should be used across the project

    pub fn incorrect_json_format() -> Self {
        ActionError {
            r#type: ActionErrorType::IncorrectJSONFormat,
            message: "Incorrect JSON format.".to_string(),
            errors: None
        }
    }

    pub fn unexpected_input_root_type<'a>(expected: impl AsRef<str>) -> Self {
        ActionError {
            r#type: ActionErrorType::UnexpectedInputRootType,
            message: format!("Unexpected root input type. Expect {}.", expected.as_ref()),
            errors: None
        }
    }

    pub fn unexpected_input_type<'a>(expected: impl Into<String>, key_path: impl AsRef<KeyPath<'a>>) -> Self {
        ActionError {
            r#type: ActionErrorType::UnexpectedInputType,
            message: "Unexpected input type found.".to_string(),
            errors: Some(hashmap!{key_path.as_ref().to_string() => format!("Expect {}.", expected.into())}),
        }
    }

    pub fn unexpected_input_key<'a>(unexpected: impl Into<String>, key_path: impl AsRef<KeyPath<'a>>) -> Self {
        ActionError {
            r#type: ActionErrorType::UnexpectedInputKey,
            message: "Unexpected key found.".to_string(),
            errors: Some(hashmap!{key_path.as_ref().to_string() => format!("Unexpected key '{}'.", unexpected.into())}),
        }
    }

    pub fn unexpected_input_value<'a>(expected: impl Into<String>, key_path: impl AsRef<KeyPath<'a>>) -> Self {
        ActionError {
            r#type: ActionErrorType::UnexpectedInputValue,
            message: "Unexpected value found.".to_string(),
            errors: Some(hashmap!{key_path.as_ref().to_string() => format!("Expect `{}'.", expected.into())}),
        }
    }

    pub fn unexpected_input_value_validation<'a>(reason: impl Into<String>, key_path: impl AsRef<KeyPath<'a>>) -> Self {
        ActionError {
            r#type: ActionErrorType::UnexpectedInputValue,
            message: "Unexpected value found.".to_string(),
            errors: Some(hashmap!{key_path.as_ref().to_string() => format!("{}", reason.into())}),
        }
    }

    pub fn missing_required_input<'a>(expected: impl Into<String>, key_path: impl AsRef<KeyPath<'a>>) -> Self {
        ActionError {
            r#type: ActionErrorType::MissingRequiredInput,
            message: "Missing required input.".to_string(),
            errors: Some(hashmap!{key_path.as_ref().to_string() => format!("Expect `{}'.", expected.into())})
        }
    }

    pub fn unexpected_object_length<'a>(expected: usize, key_path: impl AsRef<KeyPath<'a>>) -> Self {
        ActionError {
            r#type: ActionErrorType::UnexpectedObjectLength,
            message: "Unexpected object length.".to_string(),
            errors: Some(hashmap!{key_path.as_ref().to_string() => format!("Expect length {}.", expected)})
        }
    }

    pub fn invalid_key(unexpected_key: impl AsRef<str>, model: &Model) -> Self {
        ActionError {
            r#type: ActionErrorType::InvalidKey,
            message: format!("Invalid key '{}' accessed on model `{}'", unexpected_key.as_ref(), model.name()),
            errors: None
        }
    }

    pub fn permission_denied(action: impl AsRef<str>) -> Self {
        ActionError {
            r#type: ActionErrorType::PermissionDenied,
            message: format!("Permission denied for `{}'.", action.as_ref()),
            errors: None
        }
    }
}

impl Display for ActionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.r#type.fmt(f)
    }
}

impl Error for ActionError { }

unsafe impl Sync for ActionError {}
unsafe impl Send for ActionError {}
