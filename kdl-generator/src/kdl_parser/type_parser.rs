use std::sync::Arc;

use super::schema::{ArraySeparator, DataType};

pub fn generic_parse(input: &str) -> Result<DataType, String> {
    let input = input.trim();

    match input {
        "i32" | "int" => Ok(DataType::I32),
        "u32" | "uint" => Ok(DataType::U32),
        "i64" | "long" => Ok(DataType::I64),
        "u64" | "ulong" => Ok(DataType::U64),
        "f32" | "float" => Ok(DataType::F32),
        "f64" | "double" => Ok(DataType::F64),
        "bool" => Ok(DataType::Bool),
        "str" => Ok(DataType::String),
        "datetime" => Ok(DataType::Datetime),
        "json" => Ok(DataType::Json),

        _ => {
            if let Some(res) = parse_map(input) {
                res
            } else if let Some(res) = parse_tuple(input) {
                res
            } else if let Some(res) = parse_array(input) {
                res
            } else {
                Ok(DataType::Custom(input.to_owned()))
            }
        }
    }
}

// fn parse_function(input: &str) -> Option<Result<DataType, String>> {
//     const ARROW: &str = " -> ";
//
//     let input = input.trim();
//     if !input.contains(ARROW) {
//         return None;
//     }
//
//     let parameters: Result<Vec<_>, String> = input
//         .split(ARROW)
//         .filter(|c| !c.is_empty())
//         .map(generic_parse)
//         .collect();
//
//     let Ok(parameters) = parameters else {
//         return Some(Err(
//             parameters.expect_err("just checked it contains an error.")
//         ));
//     };
//
//     if parameters.is_empty() {
//         return None;
//     }
//
//     if parameters.len() == 1 {
//         return Some(Err(
//             "Input or output of function type not provided.".to_owned()
//         ));
//     }
//
//     if parameters.len() == 3 {
//         return Some(Err(
//             "Too many arrows for function type, use `(tuples)` to allow multiple parameters."
//                 .to_owned(),
//         ));
//     }
//
//     Some(Ok(DataType::Function {
//         input: Arc::new(parameters[0].clone()),
//         output: Arc::new(parameters[1].clone()),
//     }))
// }

fn parse_map(input: &str) -> Option<Result<DataType, String>> {
    const ARROW: &str = " => ";

    let input = input.trim();
    if !input.contains(ARROW) {
        return None;
    }

    let parameters: Result<Vec<_>, String> = input
        .split(ARROW)
        .filter(|c| !c.is_empty())
        .map(generic_parse)
        .collect();

    let Ok(parameters) = parameters else {
        return Some(Err(
            parameters.expect_err("just checked it contains an error.")
        ));
    };

    if parameters.is_empty() {
        return None;
    }

    if parameters.len() == 1 {
        return Some(Err("Key or value of map type not provided.".to_owned()));
    }

    if parameters.len() == 3 {
        return Some(Err(
            "Too many arrows for map type, use `(tuples)` to allow multiple parameters.".to_owned(),
        ));
    }

    Some(Ok(DataType::Map {
        key: Arc::new(parameters[0].clone()),
        value: Arc::new(parameters[1].clone()),
    }))
}

fn parse_tuple(input: &str) -> Option<Result<DataType, String>> {
    let input = input.trim();
    if !input.starts_with('(') {
        return None;
    }

    if !input.ends_with(')') {
        return Some(Err(
            "Tuple type must start with '(' and end with ')'.".to_owned()
        ));
    }

    let words: Result<Vec<_>, _> = input[1..(input.len() - 1)]
        .split(", ")
        .map(generic_parse)
        .collect();

    match words {
        Ok(words) => Some(Ok(DataType::Tuple(words))),
        Err(e) => Some(Err(e)),
    }
}

fn parse_array(input: &str) -> Option<Result<DataType, String>> {
    let input = input.trim();
    if !input.starts_with('[') {
        return None;
    }

    if !input.ends_with(']') {
        return Some(Err(format!(
            "List type must start with '[' and end with ']', found `{input}`."
        )));
    }

    let words = &input[1..(input.len() - 1)];

    // Try to figure out if we have a separator.
    let (words, separator) = match words.chars().last() {
        Some(',') => (&words[0..(words.len() - 1)], Some(ArraySeparator::Comma)),
        Some('@') => (&words[0..(words.len() - 1)], Some(ArraySeparator::At)),
        Some(':') => (&words[0..(words.len() - 1)], Some(ArraySeparator::Colon)),
        _ => (words, None),
    };

    let val = generic_parse(words);

    match (val, separator) {
        (Ok(DataType::Json), _) => Some(Ok(DataType::JsonArray { type_hint: None })),
        (Ok(val), Some(separator)) => Some(Ok(DataType::Array {
            inner: Arc::new(val),
            separator,
        })),
        (Ok(val), None) => Some(Ok(DataType::SingleElementArray(Arc::new(val)))),
        (Err(e), _) => Some(Err(e)),
    }
}
