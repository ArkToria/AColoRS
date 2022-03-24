use serde_json::Value;

pub fn check_is_default_and_delete(value: &mut Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(b) => !*b,
        Value::Number(n) => n.as_u64().unwrap_or(0) == 0,
        Value::String(s) => s.is_empty(),
        Value::Array(value_vec) => {
            value_vec.iter_mut().for_each(|v| {
                if let Value::Object(_) = v {
                    check_is_default_and_delete(v);
                }
            });

            value_vec.is_empty()
        }
        Value::Object(map) => {
            let mut contain_non_default_member = true;
            map.retain(|key, value| {
                if key == "stats" {
                    return true;
                }
                if check_is_default_and_delete(value) {
                    false
                } else {
                    contain_non_default_member = false;
                    true
                }
            });

            map.is_empty() || contain_non_default_member
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
