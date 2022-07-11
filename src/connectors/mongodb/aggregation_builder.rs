use std::collections::HashSet;

use serde_json::{Value as JsonValue, Map as JsonMap, json};
use bson::{Bson, DateTime as BsonDateTime, doc, Document, oid::ObjectId, Regex as BsonRegex};
use chrono::{Date, NaiveDate, Utc, DateTime};
use crate::core::field_type::FieldType;
use crate::core::graph::Graph;
use crate::core::input_decoder::{input_to_vec, one_length_json_obj};
use crate::core::model::{Model, ModelIndexType};
use crate::core::value::Value;
use crate::error::ActionError;


#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) enum QueryPipelineType {
    Unique,
    First,
    Many
}

pub trait ToBsonValue {
    fn to_bson_value(&self) -> Bson;
}

impl ToBsonValue for Value {
    fn to_bson_value(&self) -> Bson {
        match self {
            Value::Null => {
                Bson::Null
            }
            Value::ObjectId(val) => {
                Bson::ObjectId(ObjectId::parse_str(val.as_str()).unwrap())
            }
            Value::Bool(val) => {
                Bson::Boolean(*val)
            }
            Value::I8(val) => {
                Bson::Int32(*val as i32)
            }
            Value::I16(val) => {
                Bson::Int32(*val as i32)
            }
            Value::I32(val) => {
                Bson::Int32(*val)
            }
            Value::I64(val) => {
                Bson::Int64(*val)
            }
            Value::I128(val) => {
                Bson::Int64(*val as i64)
            }
            Value::U8(val) => {
                Bson::Int32(*val as i32)
            }
            Value::U16(val) => {
                Bson::Int32(*val as i32)
            }
            Value::U32(val) => {
                Bson::Int64(*val as i64)
            }
            Value::U64(val) => {
                Bson::Int64(*val as i64)
            }
            Value::U128(val) => {
                Bson::Int64(*val as i64)
            }
            Value::F32(val) => {
                Bson::from(val)
            }
            Value::F64(val) => {
                Bson::from(val)
            }
            Value::String(val) => {
                Bson::String(val.clone())
            }
            Value::Decimal(_val) => {
                todo!()
            }
            Value::Date(val) => {
                Bson::DateTime(BsonDateTime::parse_rfc3339_str(val.format("%Y-%m-%d").to_string()).unwrap())
            }
            Value::DateTime(val) => {
                Bson::DateTime(BsonDateTime::from(*val))
            }
            Value::Vec(val) => {
                Bson::Array(val.iter().map(|i| { i.to_bson_value() }).collect())
            }
            Value::Map(val) => {
                let mut doc = doc!{};
                for (k, v) in val {
                    doc.insert(k.to_string(), v.to_bson_value());
                }
                Bson::Document(doc)
            }
            Value::Object(_obj) => {
                panic!()
            }
        }
    }
}

fn parse_object_id(value: &JsonValue) -> Result<Bson, ActionError> {
    match value.as_str() {
        Some(val) => {
            match ObjectId::parse_str(val) {
                Ok(oid) => {
                    Ok(Bson::ObjectId(oid))
                }
                Err(_) => {
                    Err(ActionError::wrong_input_type())
                }
            }
        }
        None => {
            Err(ActionError::wrong_input_type())
        }
    }
}

fn has_i_mode(map: &JsonMap<String, JsonValue>) -> bool {
    match map.get("mode") {
        Some(val) => {
            if val.is_string() {
                return val.as_str().unwrap() == "caseInsensitive"
            } else {
                false
            }
        }
        None => {
            false
        }
    }
}

fn parse_string(value: &JsonValue) -> Result<Bson, ActionError> {
    match value.as_str() {
        Some(val) => {
            Ok(Bson::String(val.to_string()))
        }
        None => {
            Err(ActionError::wrong_input_type())
        }
    }
}

fn parse_bool(value: &JsonValue) -> Result<Bson, ActionError> {
    match value.as_bool() {
        Some(val) => {
            Ok(Bson::Boolean(val))
        }
        None => {
            Err(ActionError::wrong_input_type())
        }
    }
}

fn parse_i64(value: &JsonValue) -> Result<Bson, ActionError> {
    if value.is_i64() {
        Ok(Bson::Int64(value.as_i64().unwrap()))
    } else if value.is_u64() {
        Ok(Bson::Int64(value.as_u64().unwrap() as i64))
    } else if value.is_f64() {
        Ok(Bson::Int64(value.as_f64().unwrap() as i64))
    } else {
        Err(ActionError::wrong_input_type())
    }
}

fn parse_f64(value: &JsonValue) -> Result<Bson, ActionError> {
    if value.is_i64() {
        Ok(Bson::Double(value.as_i64().unwrap() as f64))
    } else if value.is_u64() {
        Ok(Bson::Double(value.as_u64().unwrap() as f64))
    } else if value.is_f64() {
        Ok(Bson::Double(value.as_f64().unwrap()))
    } else {
        Err(ActionError::wrong_input_type())
    }
}

fn parse_date(value: &JsonValue) -> Result<Bson, ActionError> {
    if value.is_string() {
        match NaiveDate::parse_from_str(&value.as_str().unwrap(), "%Y-%m-%d") {
            Ok(naive_date) => {
                let date: Date<Utc> = Date::from_utc(naive_date, Utc);
                let val = Value::Date(date);
                Ok(val.to_bson_value())
            }
            Err(_) => {
                Err(ActionError::wrong_date_format())
            }
        }
    } else {
        Err(ActionError::wrong_input_type())
    }
}

fn parse_datetime(value: &JsonValue) -> Result<Bson, ActionError> {
    if value.is_string() {
        match DateTime::parse_from_rfc3339(&value.as_str().unwrap()) {
            Ok(fixed_offset_datetime) => {
                let datetime: DateTime<Utc> = fixed_offset_datetime.with_timezone(&Utc);
                let value = Value::DateTime(datetime);
                Ok(value.to_bson_value())
            }
            Err(_) => {
                Err(ActionError::wrong_datetime_format())
            }
        }
    } else {
        Err(ActionError::wrong_input_type())
    }
}

fn parse_enum(value: &JsonValue, enum_name: &str, graph: &Graph) -> Result<Bson, ActionError> {
    if value.is_string() {
        let str = value.as_str().unwrap();
        let r#enum = graph.r#enum(enum_name);
        if r#enum.contains(&str.to_string()) {
            Ok(Bson::String(str.to_string()))
        } else {
            Err(ActionError::undefined_enum_value())
        }
    } else {
        Err(ActionError::wrong_input_type())
    }
}

fn parse_bson_where_entry(field_type: &FieldType, value: &JsonValue, graph: &Graph) -> Result<Bson, ActionError> {
    return match field_type {
        FieldType::Undefined => {
            panic!()
        }
        FieldType::ObjectId => {
            if value.is_string() {
                parse_object_id(value)
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let oid = parse_object_id(value)?;
                            result.insert("$eq", oid);
                        }
                        "not" => {
                            let oid = parse_object_id(value)?;
                            result.insert("$ne", oid);
                        }
                        "gt" => {
                            let oid = parse_object_id(value)?;
                            result.insert("$gt", oid);
                        }
                        "gte" => {
                            let oid = parse_object_id(value)?;
                            result.insert("$gte", oid);
                        }
                        "lt" => {
                            let oid = parse_object_id(value)?;
                            result.insert("$lt", oid);
                        }
                        "lte" => {
                            let oid = parse_object_id(value)?;
                            result.insert("$lte", oid);
                        }
                        "in" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_object_id(val)?);
                                    }
                                    result.insert("$in", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "notIn" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_object_id(val)?);
                                    }
                                    result.insert("$nin", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::Bool => {
            if value.is_boolean() {
                Ok(Bson::Boolean(value.as_bool().unwrap()))
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let b = parse_bool(value)?;
                            result.insert("$eq", b);
                        }
                        "not" => {
                            let b = parse_bool(value)?;
                            result.insert("$ne", b);
                        }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::I64 | FieldType::I128 | FieldType::U8 | FieldType::U16 | FieldType::U32 | FieldType::U64 | FieldType::U128 => {
            if value.is_i64() {
                Ok(Bson::Int64(value.as_i64().unwrap()))
            } else if value.is_u64() {
                Ok(Bson::Int64(value.as_u64().unwrap() as i64))
            } else if value.is_f64() {
                Ok(Bson::Int64(value.as_f64().unwrap() as i64))
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let b = parse_i64(value)?;
                            result.insert("$eq", b);
                        }
                        "not" => {
                            let b = parse_i64(value)?;
                            result.insert("$ne", b);
                        }
                        "gt" => {
                            let oid = parse_i64(value)?;
                            result.insert("$gt", oid);
                        }
                        "gte" => {
                            let oid = parse_i64(value)?;
                            result.insert("$gte", oid);
                        }
                        "lt" => {
                            let oid = parse_i64(value)?;
                            result.insert("$lt", oid);
                        }
                        "lte" => {
                            let oid = parse_i64(value)?;
                            result.insert("$lte", oid);
                        }
                        "in" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_i64(val)?);
                                    }
                                    result.insert("$in", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "notIn" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_i64(val)?);
                                    }
                                    result.insert("$nin", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::F32 | FieldType::F64 => {
            if value.is_i64() {
                Ok(Bson::Double(value.as_i64().unwrap() as f64))
            } else if value.is_u64() {
                Ok(Bson::Double(value.as_u64().unwrap() as f64))
            } else if value.is_f64() {
                Ok(Bson::Double(value.as_f64().unwrap()))
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let b = parse_f64(value)?;
                            result.insert("$eq", b);
                        }
                        "not" => {
                            let b = parse_f64(value)?;
                            result.insert("$ne", b);
                        }
                        "gt" => {
                            let oid = parse_f64(value)?;
                            result.insert("$gt", oid);
                        }
                        "gte" => {
                            let oid = parse_f64(value)?;
                            result.insert("$gte", oid);
                        }
                        "lt" => {
                            let oid = parse_f64(value)?;
                            result.insert("$lt", oid);
                        }
                        "lte" => {
                            let oid = parse_f64(value)?;
                            result.insert("$lte", oid);
                        }
                        "in" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_f64(val)?);
                                    }
                                    result.insert("$in", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "notIn" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_f64(val)?);
                                    }
                                    result.insert("$nin", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::Decimal => {
            todo!()
        }
        FieldType::String => {
            if value.is_string() {
                Ok(Bson::String(value.as_str().unwrap().to_string()))
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let b = parse_string(value)?;
                            result.insert("$eq", b);
                        }
                        "not" => {
                            let b = parse_string(value)?;
                            result.insert("$ne", b);
                        }
                        "gt" => {
                            let oid = parse_string(value)?;
                            result.insert("$gt", oid);
                        }
                        "gte" => {
                            let oid = parse_string(value)?;
                            result.insert("$gte", oid);
                        }
                        "lt" => {
                            let oid = parse_string(value)?;
                            result.insert("$lt", oid);
                        }
                        "lte" => {
                            let oid = parse_string(value)?;
                            result.insert("$lte", oid);
                        }
                        "in" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_string(val)?);
                                    }
                                    result.insert("$in", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "notIn" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_string(val)?);
                                    }
                                    result.insert("$nin", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "contains" => {
                            let bson_regex = BsonRegex {
                                pattern: regex::escape(parse_string(value)?.as_str().unwrap()),
                                options: if has_i_mode(map) { "i".to_string() } else { "".to_string() }
                            };
                            let regex = Bson::RegularExpression(bson_regex);
                            result.insert("$regex", regex);
                        }
                        "startsWith" => {
                            let bson_regex = BsonRegex {
                                pattern: "^".to_string() + &*regex::escape(parse_string(value)?.as_str().unwrap()),
                                options: if has_i_mode(map) { "i".to_string() } else { "".to_string() }
                            };
                            let regex = Bson::RegularExpression(bson_regex);
                            result.insert("$regex", regex);
                        }
                        "endsWith" => {
                            let bson_regex = BsonRegex {
                                pattern: regex::escape(parse_string(value)?.as_str().unwrap()) + "$",
                                options: if has_i_mode(map) { "i".to_string() } else { "".to_string() }
                            };
                            let regex = Bson::RegularExpression(bson_regex);
                            result.insert("$regex", regex);
                        }
                        "matches" => {
                            let bson_regex = BsonRegex {
                                pattern: parse_string(value)?.as_str().unwrap().to_string(),
                                options: if has_i_mode(map) { "i".to_string() } else { "".to_string() }
                            };
                            let regex = Bson::RegularExpression(bson_regex);
                            result.insert("$regex", regex);
                        }
                        "mode" => { }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::Date => {
            if value.is_string() {
                parse_date(value)
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let b = parse_date(value)?;
                            result.insert("$eq", b);
                        }
                        "not" => {
                            let b = parse_date(value)?;
                            result.insert("$ne", b);
                        }
                        "gt" => {
                            let oid = parse_date(value)?;
                            result.insert("$gt", oid);
                        }
                        "gte" => {
                            let oid = parse_date(value)?;
                            result.insert("$gte", oid);
                        }
                        "lt" => {
                            let oid = parse_date(value)?;
                            result.insert("$lt", oid);
                        }
                        "lte" => {
                            let oid = parse_date(value)?;
                            result.insert("$lte", oid);
                        }
                        "in" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_date(val)?);
                                    }
                                    result.insert("$in", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "notIn" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_date(val)?);
                                    }
                                    result.insert("$nin", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::DateTime => {
            if value.is_string() {
                parse_datetime(value)
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let b = parse_datetime(value)?;
                            result.insert("$eq", b);
                        }
                        "not" => {
                            let b = parse_datetime(value)?;
                            result.insert("$ne", b);
                        }
                        "gt" => {
                            let oid = parse_datetime(value)?;
                            result.insert("$gt", oid);
                        }
                        "gte" => {
                            let oid = parse_datetime(value)?;
                            result.insert("$gte", oid);
                        }
                        "lt" => {
                            let oid = parse_datetime(value)?;
                            result.insert("lt", oid);
                        }
                        "lte" => {
                            let oid = parse_datetime(value)?;
                            result.insert("$lte", oid);
                        }
                        "in" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_datetime(val)?);
                                    }
                                    result.insert("$in", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "notIn" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for val in arr_val {
                                        arr.push(parse_datetime(val)?);
                                    }
                                    result.insert("$nin", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::Enum(enum_name) => {
            if value.is_string() {
                parse_enum(value, enum_name, graph)
            } else if value.is_object() {
                let map = value.as_object().unwrap();
                let mut result = doc!{};
                for (key, value) in map {
                    match key.as_str() {
                        "equals" => {
                            let b = parse_enum(value, enum_name, graph)?;
                            result.insert("$eq", b);
                        }
                        "not" => {
                            let b = parse_enum(value, enum_name, graph)?;
                            result.insert("$ne", b);
                        }
                        "in" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for _val in arr_val {
                                        arr.push(parse_enum(value, enum_name, graph)?);
                                    }
                                    result.insert("$in", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        "notIn" => {
                            match value.as_array() {
                                Some(arr_val) => {
                                    let mut arr: Vec<Bson> = Vec::new();
                                    for _val in arr_val {
                                        arr.push(parse_enum(value, enum_name, graph)?);
                                    }
                                    result.insert("$nin", arr);
                                }
                                None => {
                                    return Err(ActionError::wrong_input_type());
                                }
                            }
                        }
                        &_ => {
                            return Err(ActionError::wrong_input_type());
                        }
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::Vec(inner_field) => {
            if value.is_object() {
                let mut result = doc!{};
                let (key, matcher) = one_length_json_obj(value, "")?;
                match key {
                    "has" => {
                        let inner = parse_bson_where_entry(&inner_field.field_type, matcher, graph)?;
                        if inner.as_document().is_some() {
                            result.insert("$elemMatch", inner);
                        } else {
                            return Ok(inner);
                        }
                    }
                    "hasEvery" => {
                        if !matcher.is_array() {
                            return Err(ActionError::wrong_input_type());
                        }
                        let matcher = matcher.as_array().unwrap();
                        let inner = matcher.iter().map(|v| {
                            parse_bson_where_entry(&inner_field.field_type, v, graph).unwrap()
                        }).collect::<Vec<Bson>>();
                        result.insert("$all", inner);
                    }
                    "hasSome" => {
                        if !matcher.is_array() {
                            return Err(ActionError::wrong_input_type());
                        }
                        let matcher = matcher.as_array().unwrap();
                        let inner = matcher.iter().map(|v| {
                            parse_bson_where_entry(&inner_field.field_type, v, graph).unwrap()
                        }).collect::<Vec<Bson>>();
                        result.insert("$in", inner);
                    }
                    "isEmpty" => {
                        if matcher.is_boolean() && (matcher.as_bool().unwrap() == true) {
                            result.insert("$size", 0);
                        }
                    }
                    "length" => {
                        let ft = FieldType::U64;
                        let num = parse_bson_where_entry(&ft, matcher, graph).unwrap();
                        result.insert("$size", num);
                    }
                    "equals" => {
                        if !matcher.is_array() {
                            return Err(ActionError::wrong_input_type());
                        }
                        let matcher = matcher.as_array().unwrap();
                        let inner = matcher.iter().map(|v| {
                            parse_bson_where_entry(&inner_field.field_type, v, graph).unwrap()
                        }).collect::<Vec<Bson>>();
                        result.insert("$eq", inner);
                    }
                    _ => {
                        return Err(ActionError::wrong_input_type());
                    }
                }
                Ok(Bson::Document(result))
            } else {
                Err(ActionError::wrong_input_type())
            }
        }
        FieldType::Map(_) => {
            panic!()
        }
        FieldType::Object(_) => {
            panic!()
        }
    }
}

pub(crate) fn build_unsets_for_match_lookup(model: &Model, _graph: &Graph, r#where: Option<&JsonValue>) -> Result<Vec<Document>, ActionError> {
    if let None = r#where { return Ok(vec![]); }
    let r#where = r#where.unwrap();
    if !r#where.is_object() { return Err(ActionError::invalid_query_input("'where' should be an object.")); }
    let r#where = r#where.as_object().unwrap();
    let mut retval: Vec<Document> = vec![];
    for (key, _value) in r#where.iter() {
        let relation = model.relation(key);
        if relation.is_some() {
            retval.push(doc!{"$unset": key})
        }
    }
    Ok(retval)
}

pub(crate) fn build_match_prediction_lookup(model: &Model, graph: &Graph, r#where: Option<&JsonValue>) -> Result<Vec<Document>, ActionError> {
    if let None = r#where { return Ok(vec![]); }
    let r#where = r#where.unwrap();
    if !r#where.is_object() { return Err(ActionError::invalid_query_input("'where' should be an object.")); }
    let r#where = r#where.as_object().unwrap();
    let mut include_input = JsonMap::new();
    for (key, value) in r#where.iter() {
        let relation = model.relation(key);
        if relation.is_some() {
            let (command, r_where) = one_length_json_obj(value, "")?;
            match command {
                "some" | "is" => {
                    include_input.insert(key.to_string(), json!({
                        "where": r_where,
                        "take": 1
                    }));
                }
                "none" | "isNot" => {
                    include_input.insert(key.to_string(), json!({
                        "where": r_where,
                        "take": 1
                    }));
                }
                "all" => {
                    include_input.insert(key.to_string(), json!({
                        "where": {"NOT": r_where},
                        "take": 1
                    }));
                }
                _ => {}
            }
        }
    }
    Ok(if !include_input.is_empty() {
        build_lookup_inputs(model, graph, QueryPipelineType::Many, false, &JsonValue::Object(include_input))?
    } else {
        vec![]
    })
}

pub(crate) fn build_where_input(model: &Model, graph: &Graph, r#where: Option<&JsonValue>) -> Result<Document, ActionError> {
    if let None = r#where { return Ok(doc!{}); }
    let r#where = r#where.unwrap();
    if !r#where.is_object() { return Err(ActionError::invalid_query_input("'where' should be an object.")); }
    let r#where = r#where.as_object().unwrap();
    let mut doc = doc!{};
    for (key, value) in r#where.iter() {
        if key == "AND" {
            let mut vals: Vec<Document> = vec![];
            for val in value.as_array().unwrap() {
                vals.push(build_where_input(model, graph, Some(val))?);
            }
            doc.insert("$and", vals);
            continue;
        } else if key == "OR" {
            let mut vals: Vec<Document> = vec![];
            for val in value.as_array().unwrap() {
                vals.push(build_where_input(model, graph, Some(val))?);
            }
            doc.insert("$or", vals);
            continue;
        } else if key == "NOT" {
            doc.insert("$nor", vec![build_where_input(model, graph, Some(value))?]);
            continue;
        } else if !model.query_keys().contains(key) {
            return Err(ActionError::keys_unallowed());
        }
        let field = model.field(key);
        if field.is_some() {
            let field = field.unwrap();
            let db_key = field.column_name();
            let bson_result = parse_bson_where_entry(&field.field_type, value, graph);
            match bson_result {
                Ok(bson) => {
                    doc.insert(db_key, bson);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        } else {
            let relation = model.relation(key).unwrap();
            let model_name = &relation.model;
            let this_model = graph.model(model_name)?;
            let (command, inner_where) = one_length_json_obj(value, "")?;
            let _inner_where = build_where_input(this_model, graph, Some(inner_where))?;
            match command {
                "none" | "isNot" => {
                    doc.insert(key, doc!{"$size": 0});
                }
                "some" | "is" => {
                    doc.insert(key, doc!{"$size": 1});
                }
                "all" => {
                    doc.insert(key, doc!{"$size": 0});
                }
                _ => {

                }
            }
        }
    }
    Ok(doc)
}

pub(crate) fn build_order_by_input(_model: &Model, _graph: &Graph, order_by: Option<&JsonValue>, reverse: bool) -> Result<Document, ActionError> {
    if order_by.is_none() {
        return Ok(doc!{});
    }
    let order_by = order_by.unwrap();
    if !order_by.is_object() && !order_by.is_array() {
        return Err(ActionError::invalid_query_input("Order by inputs should be an object or an array of objects."));
    }
    let order_by = input_to_vec(order_by)?;
    let mut retval = doc!{};
    for sort in order_by {
        let (key, value) = one_length_json_obj(sort, "")?;
        if value.is_string() {
            let str_val = value.as_str().unwrap();
            if str_val == "asc" {
                retval.insert(key, if reverse { -1 } else { 1 });
            } else if str_val == "desc" {
                retval.insert(key, if reverse { 1 } else { -1 });
            }
        } else {
            return Err(ActionError::invalid_query_input("Order by input value should be whether string 'asc' or 'desc'."));
        }
    }
    Ok(retval)
}

fn build_select_input(model: &Model, graph: &Graph, select: &JsonValue) -> Result<Option<Document>, ActionError> {
    let mut true_list: Vec<&str> = vec![];
    let mut false_list: Vec<&str> = vec![];
    let map = select.as_object().unwrap();
    for (key, value) in map {
        let bool_value = value.as_bool().unwrap();
        if bool_value {
            true_list.push(key.as_str());
        } else {
            false_list.push(key.as_str());
        }
    }
    let true_empty = true_list.is_empty();
    let false_empty = false_list.is_empty();
    if true_empty && false_empty {
        // just do nothing
        return Ok(None);
    } else if !false_empty {
        // all - false
        let primary_names = model.primary().items.iter().map(|i| i.field_name.clone()).collect::<Vec<String>>();
        let mut result = doc!{};
        model.all_keys().iter().for_each(|k| {
            let field = model.field(k);
            if let Some(field) = field {
                let db_name = field.column_name();
                if primary_names.contains(k) {
                    result.insert(db_name, 1);
                } else {
                    if !false_list.contains(&&***&k) {
                        result.insert(db_name, 1);
                    }
                }
            }
        });
        return Ok(Some(result));
    } else {
        // true
        let primary_names = model.primary().items.iter().map(|i| i.field_name.clone()).collect::<Vec<String>>();
        let mut result = doc!{};
        model.all_keys().iter().for_each(|k| {
            let field = model.field(k);
            if let Some(field) = field {
                let db_name = field.column_name();
                if primary_names.contains(k) {
                    result.insert(db_name, 1);
                } else {
                    if true_list.contains(&&***&k) {
                        result.insert(db_name, 1);
                    }
                }
            }
        });
        return Ok(Some(result));
    }
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
        let model_name = model.name();
        return Err(ActionError::invalid_query_input(format!("'include' on model '{model_name}' is not an object. Please check your input.")));
    }
    let include = include.unwrap();
    let mut retval: Vec<Document> = vec![];
    for (key, value) in include.iter() {
        let relation = model.relation(key);
        if relation.is_none() {
            let model_name = &model.name();
            return Err(ActionError::invalid_query_input(format!("Relation '{key}' on model '{model_name}' is not exist. Please check your input.")));
        }
        let relation = relation.unwrap();
        let relation_name = &relation.name;
        let relation_model_name = &relation.model;
        let relation_model = graph.model(relation_model_name)?;
        if value.is_boolean() || value.is_object() {
            if relation.through.is_none() { // without join table
                let mut let_value = doc!{};
                let mut eq_values: Vec<Document> = vec![];
                for (index, field_name) in relation.fields.iter().enumerate() {
                    let field_name = model.field(field_name).unwrap().column_name();
                    let reference_name = relation.references.get(index).unwrap();
                    let reference_name_column_name = relation_model.field(reference_name).unwrap().column_name();
                    let_value.insert(reference_name, format!("${field_name}"));
                    eq_values.push(doc!{"$eq": [format!("${reference_name_column_name}"), format!("$${reference_name}")]});
                }
                let mut inner_pipeline = if value.is_object() {
                    build_query_pipeline_from_json(relation_model, graph, r#type, mutation_mode, value)?
                } else {
                    vec![]
                };
                let inner_is_reversed = has_negative_take(value);
                let inner_match = inner_pipeline.iter().find(|v| v.get("$match").is_some());
                let has_inner_match = inner_match.is_some();
                let mut inner_match = if has_inner_match {
                    inner_match.unwrap().clone()
                } else {
                    doc!{"$match": {}}
                };
                let inner_match_inner = inner_match.get_mut("$match").unwrap().as_document_mut().unwrap();
                if inner_match_inner.get("$expr").is_none() {
                    inner_match_inner.insert("$expr", doc!{});
                }
                if inner_match_inner.get("$expr").unwrap().as_document().unwrap().get("$and").is_none() {
                    inner_match_inner.get_mut("$expr").unwrap().as_document_mut().unwrap().insert("$and", vec![] as Vec<Document>);
                }
                inner_match_inner.get_mut("$expr").unwrap().as_document_mut().unwrap().get_mut("$and").unwrap().as_array_mut().unwrap().extend(eq_values.iter().map(|item| Bson::Document(item.clone())));
                if has_inner_match {
                    let index = inner_pipeline.iter().position(|v| v.get("$match").is_some()).unwrap();
                    inner_pipeline.remove(index);
                    inner_pipeline.insert(index, inner_match);
                } else {
                    inner_pipeline.insert(0, inner_match);
                }
                let lookup = doc!{"$lookup": {
                    "from": &relation_model.table_name(),
                    "as": key,
                    "let": let_value,
                    "pipeline": inner_pipeline
                }};
                retval.push(lookup);
                if inner_is_reversed {
                    retval.push(doc!{"$set": {relation_name: {"$reverseArray": format!("${relation_name}")}}});
                }
            } else { // with join table
                let join_model = graph.model(relation.through.as_ref().unwrap())?;
                let local_relation_on_join_table = join_model.relation(relation.fields.get(0).unwrap()).unwrap();
                let foreign_relation_on_join_table = join_model.relation(relation.references.get(0).unwrap()).unwrap();
                let foreign_model_name = &foreign_relation_on_join_table.model;
                let foreign_model = graph.model(foreign_model_name)?;
                let mut outer_let_value = doc!{};
                let mut outer_eq_values: Vec<Document> = vec![];
                let mut inner_let_value = doc!{};
                let mut inner_eq_values: Vec<Document> = vec![];
                for (index, join_table_field_name) in local_relation_on_join_table.fields.iter().enumerate() {
                    let local_unique_field_name = local_relation_on_join_table.references.get(index).unwrap();
                    let local_unique_field_column_name = model.field(local_unique_field_name).unwrap().column_name();
                    outer_let_value.insert(join_table_field_name, format!("${local_unique_field_column_name}"));
                    outer_eq_values.push(doc!{"$eq": [format!("${join_table_field_name}"), format!("$${join_table_field_name}")]});
                }
                for (index, join_table_reference_name) in foreign_relation_on_join_table.fields.iter().enumerate() {
                    let foreign_unique_field_name = foreign_relation_on_join_table.references.get(index).unwrap();
                    let foreign_unique_field_column_name = foreign_model.field(foreign_unique_field_name).unwrap().column_name();
                    inner_let_value.insert(join_table_reference_name, format!("${join_table_reference_name}"));
                    inner_eq_values.push(doc!{"$eq": [format!("${foreign_unique_field_column_name}"), format!("$${join_table_reference_name}")]});
                }
                let mut original_inner_pipeline = if value.is_object() {
                    build_query_pipeline_from_json(foreign_model, graph, QueryPipelineType::Many, false, value)?
                } else {
                    vec![]
                };
                let inner_is_reversed = has_negative_take(value);
                let original_inner_pipeline_immu = original_inner_pipeline.clone();
                let mut inner_match = doc!{
                    "$expr": {
                        "$and": inner_eq_values
                    }
                };
                let original_inner_match = original_inner_pipeline.iter().find(|v| {
                    v.get("$match").is_some()
                });
                if original_inner_match.is_some() {
                    let original_inner_match = original_inner_match.unwrap();
                    let doc = original_inner_match.get_document("$match").unwrap();
                    for (k, v) in doc.iter() {
                        inner_match.insert(k, v);
                    }
                }
                let index = original_inner_pipeline.iter().position(|v| {
                    v.get("$match").is_some()
                });
                if index.is_some() {
                    original_inner_pipeline.remove(index.unwrap());
                    original_inner_pipeline.insert(index.unwrap(), doc!{"$match": inner_match});
                } else {
                    original_inner_pipeline.insert(0, doc!{"$match": inner_match});
                }
                let original_inner_sort = original_inner_pipeline_immu.iter().find(|v| {
                    v.get("$sort").is_some()
                });
                let index = original_inner_pipeline.iter().position(|v| {
                    v.get("$sort").is_some()
                });
                if index.is_some() {
                    original_inner_pipeline.remove(index.unwrap());
                }
                let original_inner_skip = original_inner_pipeline_immu.iter().find(|v| {
                    v.get("$skip").is_some()
                });
                let index = original_inner_pipeline.iter().position(|v| {
                    v.get("$skip").is_some()
                });
                if index.is_some() {
                    original_inner_pipeline.remove(index.unwrap());
                }
                let original_inner_limit = original_inner_pipeline_immu.iter().find(|v| {
                    v.get("$limit").is_some()
                });
                let index = original_inner_pipeline.iter().position(|v| {
                    v.get("$limit").is_some()
                });
                if index.is_some() {
                    original_inner_pipeline.remove(index.unwrap());
                }
                let mut target = doc!{
                    "$lookup": {
                        "from": join_model.table_name(),
                        "as": relation_name,
                        "let": outer_let_value,
                        "pipeline": [{
                            "$match": {
                                "$expr": {
                                    "$and": outer_eq_values
                                }
                            }
                        }, {
                            "$lookup": {
                                "from": foreign_model.table_name(),
                                "as": relation_name,
                                "let": inner_let_value,
                                "pipeline": original_inner_pipeline
                            }
                        }, {
                            "$unwind": {
                                "path": format!("${relation_name}")
                            }
                        }, {
                            "$replaceRoot": {
                                "newRoot": format!("${relation_name}")
                            }
                        }]
                    }
                };
                if original_inner_sort.is_some() {
                    let original_inner_sort = original_inner_sort.unwrap();
                    target.get_document_mut("$lookup").unwrap().get_array_mut("pipeline").unwrap().push(Bson::Document(original_inner_sort.clone()));
                }
                if original_inner_skip.is_some() {
                    let original_inner_skip = original_inner_skip.unwrap();
                    target.get_document_mut("$lookup").unwrap().get_array_mut("pipeline").unwrap().push(Bson::Document(original_inner_skip.clone()));
                }
                if original_inner_limit.is_some() {
                    let original_inner_limit = original_inner_limit.unwrap();
                    target.get_document_mut("$lookup").unwrap().get_array_mut("pipeline").unwrap().push(Bson::Document(original_inner_limit.clone()));
                }
                retval.push(target);
                if inner_is_reversed {
                    retval.push(doc!{"$set": {relation_name: {"$reverseArray": format!("${relation_name}")}}});
                }
            }
        } else {
            let model_name = model.name();
            return Err(ActionError::invalid_query_input(format!("Relation '{key}' on model '{model_name}' has a unrecognized value. It's either a boolean or an object. Please check your input.")));
        }
    }
    Ok(retval)
}

fn build_query_pipeline(
    model: &Model,
    graph: &Graph,
    _type: QueryPipelineType,
    mutation_mode: bool,
    r#where: Option<&JsonValue>,
    order_by: Option<&JsonValue>,
    cursor: Option<&JsonValue>,
    take: Option<i32>,
    skip: Option<i32>,
    page_size: Option<i32>,
    page_number: Option<i32>,
    include: Option<&JsonValue>,
    select: Option<&JsonValue>,
) -> Result<Vec<Document>, ActionError> {
    // cursor tweaks things so that we validate cursor first
    let cursor_additional_where = if cursor.is_some() {
        let cursor = cursor.unwrap();
        if !cursor.is_object() {
            return Err(ActionError::invalid_query_input("'cursor' should be an object represents unique where input."));
        }
        if order_by.is_none() {
            return Err(ActionError::invalid_query_input("'cursor' should be used together with 'orderBy'."));
        }
        let order_by = order_by.unwrap();
        if !order_by.is_object() {
            return Err(ActionError::invalid_query_input("'orderBy' should be an object."));
        }
        let order_by_map = order_by.as_object().unwrap();
        if order_by_map.len() != 1 {
            return Err(ActionError::invalid_query_input("'orderBy' used with 'cursor' should have a single key which represents a unique constraint."));
        }
        let cursor_map = cursor.as_object().unwrap();
        if cursor_map.len() != 1 {
            return Err(ActionError::invalid_query_input("'cursor' should have a single key which represents a unique constraint."));
        }
        let (order_by_key, order_by_value) = one_length_json_obj(order_by, "")?;
        let (cursor_key, cursor_value) = one_length_json_obj(cursor, "")?;
        if order_by_key != cursor_key {
            return Err(ActionError::invalid_query_input("'cursor' and 'orderBy' should have single same key."));
        }
        if !order_by_value.is_string() {
            return Err(ActionError::invalid_query_input("Field value of 'orderBy' should be one of 'asc' or 'desc'."));
        }
        let order_by_str = order_by_value.as_str().unwrap();
        if order_by_str != "asc" && order_by_str != "desc" {
            return Err(ActionError::invalid_query_input("Field value of 'orderBy' should be one of 'asc' or 'desc'."));
        }
        let mut valid = false;
        for index in model.indices() {
            if index.items.len() == 1 {
                if index.index_type == ModelIndexType::Unique || index.index_type == ModelIndexType::Primary {
                    if index.items.get(0).unwrap().field_name == cursor_key {
                        valid = true;
                    }
                }
            }
        };
        let mut order_asc = order_by_str == "asc";
        if take.is_some() {
            let take = take.unwrap();
            if take < 0 {
                order_asc = !order_asc;
            }
        }
        let cursor_where_key = if order_asc { "gte" } else { "lte" };
        let cursor_additional_where = build_where_input(model, graph, Some(&json!({cursor_key: {cursor_where_key: cursor_value}})))?;
        Some(cursor_additional_where)
    } else {
        None
    };

    // $build the pipeline
    let mut retval: Vec<Document> = vec![];
    // $lookup for matching
    let lookups_for_matching = build_match_prediction_lookup(model, graph, r#where)?;
    if !lookups_for_matching.is_empty() {
        retval.extend(lookups_for_matching);
    }
    // $match
    let r#match = build_where_input(model, graph, r#where)?;
    if !r#match.is_empty() {
        if cursor_additional_where.is_some() {
            retval.push(doc!{"$match": {"$and": [r#match, cursor_additional_where.unwrap()]}});
        } else {
            retval.push(doc!{"$match": r#match});
        }
    } else {
        if cursor_additional_where.is_some() {
            retval.push(doc!{"$match": cursor_additional_where.unwrap()});
        }
    }
    // remove lookup for matching here
    let unsets = build_unsets_for_match_lookup(model, graph, r#where)?;
    if !unsets.is_empty() {
        retval.extend(unsets);
    }
    // $sort
    let reverse = match take {
        Some(take) => take < 0,
        None => false
    };
    let sort = build_order_by_input(model, graph, order_by, reverse)?;
    if !sort.is_empty() {
        retval.push(doc!{"$sort": sort});
    }
    // $skip and $limit
    if page_size.is_some() && page_number.is_some() {
        retval.push(doc!{"$skip": ((page_number.unwrap() - 1) * page_size.unwrap()) as i64});
        retval.push(doc!{"$limit": page_size.unwrap() as i64});
    } else {
        if skip.is_some() {
            retval.push(doc!{"$skip": skip.unwrap() as i64});
        }
        if take.is_some() {
            retval.push(doc!{"$limit": take.unwrap().abs() as i64});
        }
    }
    // $project
    if select.is_some() {
        let select_input = build_select_input(model, graph, select.unwrap())?;
        if let Some(select_input) = select_input {
            retval.push(doc!{"$project": select_input})
        }
    }
    // $lookup
    if include.is_some() {
        let mut lookups = build_lookup_inputs(model, graph, QueryPipelineType::Many, mutation_mode, include.unwrap())?;
        if !lookups.is_empty() {
            retval.append(&mut lookups);
        }
    }
    Ok(retval)
}

fn unwrap_i32(value: Option<&JsonValue>) -> Option<i32> {
    match value {
        Some(value) => Some(value.as_i64().unwrap() as i32),
        None => None
    }
}

pub(crate) fn validate_where_unique(model: &Model, r#where: &Option<&JsonValue>) -> Result<(), ActionError> {
    if r#where.is_none() {
        return Err(ActionError::invalid_query_input("Unique query should have a where which represents unique key or keys."));
    }
    let r#where = r#where.unwrap();
    if !r#where.is_object() {
        return Err(ActionError::wrong_json_format());
    }
    let values = r#where.as_object().unwrap();
    // see if key is valid
    let set_vec: Vec<String> = values.keys().map(|k| k.clone()).collect();
    let set = HashSet::from_iter(set_vec.iter().map(|k| k.clone()));
    if !model.unique_query_keys().contains(&set) {
        return Err(ActionError::field_is_not_unique())
    }
    Ok(())
}

pub(crate) fn has_negative_take(json_value: &JsonValue) -> bool {
    if json_value.is_object() {
        let take = json_value.as_object().unwrap().get("take");
        if take.is_some() {
            let take = take.unwrap();
            if take.is_number() {
                let take = take.as_i64().unwrap();
                return take < 0;
            }
        }
    }
    false
}

/// Build MongoDB aggregation pipeline for querying.
/// # Arguments
///
/// * `mutation_mode` - When mutation mode is true, `select` and `include` is ignored.
///
pub(crate) fn build_query_pipeline_from_json(
    model: &Model,
    graph: &Graph,
    r#type: QueryPipelineType,
    mutation_mode: bool,
    json_value: &JsonValue
) -> Result<Vec<Document>, ActionError> {
    let json_value = json_value.as_object();
    if json_value.is_none() {
        return Err(ActionError::invalid_query_input("Query input should be an object."));
    }
    let json_value = json_value.unwrap();
    let r#where = json_value.get("where");
    if r#type == QueryPipelineType::Unique {
        validate_where_unique(model, &r#where)?;
    }
    let order_by = json_value.get("orderBy");
    let cursor = json_value.get("cursor");
    let take = unwrap_i32(json_value.get("take"));
    let skip = unwrap_i32(json_value.get("skip"));
    let page_number = unwrap_i32(json_value.get("pageNumber"));
    let page_size = unwrap_i32(json_value.get("pageSize"));
    let include = if !mutation_mode { json_value.get("include") } else { None };
    let select = if !mutation_mode { json_value.get("select") } else { None };
    build_query_pipeline(model, graph, r#type, mutation_mode, r#where, order_by, cursor, take, skip, page_size, page_number, include, select)
}
