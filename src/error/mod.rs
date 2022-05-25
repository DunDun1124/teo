use std::collections::HashMap;
use serde::Serialize;


#[derive(Debug, PartialEq, Serialize)]
pub enum ActionErrorType {
    KeysUnallowed,
    ActionUnrecognized,
    InvalidInput,
    WrongInputType,
    WrongDateFormat,
    WrongDateTimeFormat,
    WrongEnumChoice,
    WrongJSONFormat,
    ValueRequired,
    ValidationError,
    UnknownDatabaseWriteError,
    NotFound,
    InternalServerError,
    MissingActionName,
    UndefinedAction,
    UnallowedAction,
    MissingInputSection,
    ObjectNotFound,
}

impl ActionErrorType {
    pub fn code(&self) -> u16 {
        match self {
            ActionErrorType::KeysUnallowed => { 400 }
            ActionErrorType::ActionUnrecognized => { 400 }
            ActionErrorType::InvalidInput => { 400 }
            ActionErrorType::WrongInputType => { 400 }
            ActionErrorType::WrongDateFormat => { 400 }
            ActionErrorType::WrongDateTimeFormat => { 400 }
            ActionErrorType::WrongEnumChoice => { 400 }
            ActionErrorType::ValueRequired => { 400 }
            ActionErrorType::ValidationError => { 400 }
            ActionErrorType::WrongJSONFormat => { 400 }
            ActionErrorType::MissingActionName => { 400 }
            ActionErrorType::UndefinedAction => { 400 }
            ActionErrorType::UnallowedAction => { 400 }
            ActionErrorType::UnknownDatabaseWriteError => { 500 }
            ActionErrorType::NotFound => { 404 }
            ActionErrorType::InternalServerError => { 500 }
            ActionErrorType::MissingInputSection => { 400 }
            ActionErrorType::ObjectNotFound => { 404 }
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
    pub fn keys_unallowed() -> Self {
        ActionError {
            r#type: ActionErrorType::KeysUnallowed,
            message: "Unallowed keys detected.".to_string(),
            errors: None
        }
    }

    pub fn action_unrecognized() -> Self {
        ActionError {
            r#type: ActionErrorType::ActionUnrecognized,
            message: "This action is unrecognized.".to_string(),
            errors: None
        }
    }

    pub fn invalid_input(key: &'static str, reason: String) -> Self {
        let mut fields = HashMap::with_capacity(1);
        fields.insert(key.to_string(), reason);
        ActionError {
            r#type: ActionErrorType::InvalidInput,
            message: "Invalid value found in input values.".to_string(),
            errors: Some(fields)
        }
    }

    pub fn wrong_input_type() -> Self {
        ActionError {
            r#type: ActionErrorType::WrongInputType,
            message: "Input type is unexpected.".to_string(),
            errors: None
        }
    }

    pub fn wrong_date_format() -> Self {
        ActionError {
            r#type: ActionErrorType::WrongDateFormat,
            message: "Date format is unexpected.".to_string(),
            errors: None
        }
    }

    pub fn wrong_datetime_format() -> Self {
        ActionError {
            r#type: ActionErrorType::WrongDateTimeFormat,
            message: "Datetime format is unexpected.".to_string(),
            errors: None
        }
    }

    pub fn wrong_enum_choice() -> Self {
        ActionError {
            r#type: ActionErrorType::WrongEnumChoice,
            message: "Enum value is unexpected.".to_string(),
            errors: None
        }
    }

    pub fn value_required() -> Self {
        ActionError {
            r#type: ActionErrorType::ValueRequired,
            message: "Value is required.".to_string(),
            errors: None
        }
    }

    pub fn unique_value_duplicated(field: &str) -> Self {
        let mut errors: HashMap<String, String> = HashMap::with_capacity(1);
        errors.insert(field.to_string(), "Unique value duplicated.".to_string());
        ActionError {
            r#type: ActionErrorType::ValidationError,
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

    pub fn not_found() -> Self {
        ActionError {
            r#type: ActionErrorType::NotFound,
            message: "Not found.".to_string(),
            errors: None
        }
    }

    pub fn wrong_json_format() -> Self {
        ActionError {
            r#type: ActionErrorType::WrongJSONFormat,
            message: "Wrong JSON format.".to_string(),
            errors: None
        }
    }

    pub fn missing_action_name() -> Self {
        ActionError {
            r#type: ActionErrorType::MissingActionName,
            message: "Missing action name.".to_string(),
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

    pub fn missing_input_section() -> Self {
        ActionError {
            r#type: ActionErrorType::MissingInputSection,
            message: "Input incomplete.".to_string(),
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
}