use std::str::FromStr;

use serde_json::Number;

pub(super) fn parse_str_as_number(value: &str) -> Result<Number, <Number as FromStr>::Err> {
    if let Ok(value) = value.parse() {
        return Ok(value);
    }

    let original_str_value = value;
    let value_without_shit = value.replace([',', '%', '"'], "");

    let mut value_as_number = Number::from_str(&value_without_shit)?;
    if original_str_value.contains('%') {
        value_as_number =
            Number::from_f64(value_as_number.as_f64().map(|f| f / 100.0).unwrap()).unwrap();
    }

    Ok(value_as_number)
}
