use std::ops::ControlFlow;

use anyhow::Result;
use serde_json::Value;

pub fn to_string_ignore_default(value: &Value) -> Result<String>
where
{
    let value: &Value = &value;
    let mut output = String::new();
    match value {
        Value::Object(mp) => {
            output += "{\n";
            let mut tmp = String::new();
            let mut flag = false;
            for pair in mp {
                let (name, value) = pair;
                if is_default(value) {
                    continue;
                }
                if flag {
                    tmp += ",\n"
                }
                flag = true;

                tmp += &format!("\"{}\": {}", name, to_string_ignore_default(value)?);
            }
            output += &add_tab(tmp);
            output += "}";
        }
        Value::Array(arr) => {
            output += "[\n";
            let mut tmp = String::new();
            let mut flag = false;
            for value in arr {
                if flag {
                    tmp += ",\n"
                }
                flag = true;

                tmp += &to_string_ignore_default(value)?;
            }
            output += &add_tab(tmp);
            output += "]";
        }
        v => return Ok(v.to_string()),
    }
    Ok(output)
}

fn add_tab(string: String) -> String {
    string.lines().map(|s| format!("{}{}\n", " ", s)).collect()
}

fn is_default(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(b) => !b,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) == 0.0,
        Value::Array(a) => a.is_empty(),
        Value::String(s) => s.is_empty(),
        Value::Object(object) => {
            if object.is_empty() {
                true
            } else {
                let mut flag = true;
                for pair in object {
                    let (_, value) = pair;
                    if !is_default(value) {
                        flag = false;
                    }
                }
                flag
            }
        }
    }
}

#[test]
fn test_to_string() -> Result<()> {
    let john = serde_json::json!({
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ],
        "testnull": null,
        "testnullarray": [],
    });

    println!("first phone number: {}", john["phones"][0]);

    // Convert to a string of JSON and print it out
    println!("{}", serde_json::to_string_pretty(&john)?);
    println!("{}", to_string_ignore_default(&john)?);
    Ok(())
}
