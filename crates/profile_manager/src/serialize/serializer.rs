use serde_json::Value;

pub fn check_is_default_and_delete(value: &mut Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(b) => !*b,
        Value::Number(n) => n.as_u64().unwrap_or(0) == 0,
        Value::String(s) => s.is_empty(),
        Value::Array(value_vec) => {
            if value_vec.is_empty() {
                true
            } else {
                value_vec.iter_mut().for_each(|v| {
                    if let Value::Object(_) = v {
                        check_is_default_and_delete(v);
                    }
                });
                false
            }
        }
        Value::Object(map) => {
            if map.is_empty() {
                true
            } else {
                let mut flag = true;
                map.retain(|_, value| {
                    if !check_is_default_and_delete(value) {
                        flag = false;
                        return true;
                    }
                    false
                });
                flag
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::serialize::serializer::check_is_default_and_delete;
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
            "testnulls": {
                "testempty": {},
                "testdefault": false,
                "testzero": 0,
            },
            "testnotnulls": {
                "testempty": {},
                "testdefault": false,
                "testzero": 0,
                "testone": 1,
            },
            "nullarray": [],
            "notnullarray": [
                {}
            ],
        });

        let mut clon = john.clone();
        check_is_default_and_delete(&mut clon);
        println!("{}", serde_json::to_string_pretty(&clon)?);

        assert_eq!(
            r#"{
  "age": 43,
  "name": "John Doe",
  "notnullarray": [
    {}
  ],
  "phones": [
    "+44 1234567",
    "+44 2345678"
  ],
  "testnotnulls": {
    "testone": 1
  }
}"#,
            serde_json::to_string_pretty(&clon)?
        );
        Ok(())
    }
}
