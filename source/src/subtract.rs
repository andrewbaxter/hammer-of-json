fn recurse<
    'a,
>(
    errors: &mut Vec<String>,
    path: &mut Vec<&'a str>,
    source: &mut serde_json::Map<String, serde_json::Value>,
    other: &'a serde_json::Map<String, serde_json::Value>,
    missing_ok: bool,
) {
    for (k, other_val) in other {
        path.push(k);
        if let Some(source_val) = source.get_mut(k) {
            if source_val == other_val {
                source.remove(k);
            } else if let (serde_json::Value::Object(source), serde_json::Value::Object(other)) =
                (source_val, other_val) {
                recurse(errors, path, source, other, missing_ok);
            } else {
                // nop
            }
        } else {
            if missing_ok {
                // nop
            } else {
                errors.push(format!("Trying to subtract path [{:?}] but no value exists at that path", path));
            }
        }
        path.pop();
    }
}

pub fn subtract(source: &mut serde_json::Value, other: &serde_json::Value, missing_ok: bool) -> Result<(), String> {
    let mut layer_errors = vec![];
    if source == other {
        *source = serde_json::Value::Null;
    } else if let (serde_json::Value::Object(source), serde_json::Value::Object(other)) = (source, other) {
        recurse(&mut layer_errors, &mut vec![], source, other, missing_ok);
    } else {
        // nop
    }
    if !layer_errors.is_empty() {
        return Err(layer_errors.iter().map(|e| format!("- {}", e)).collect::<Vec<_>>().join("\n"));
    }
    return Ok(());
}

#[cfg(test)]
mod test {
    use {
        super::subtract,
        serde_json::json,
    };

    #[test]
    fn base() {
        let mut source = json!({
            "a": {
                "b": {
                    "c": 4,
                    "d": "hello",
                },
                "e": true,
            },
            "f": false,
        });
        subtract(&mut source, &json!({
            "a": {
                "b": {
                    "d": "hello",
                }
            }
        }), true).unwrap();
        assert_eq!(source, json!({
            "a": {
                "b": {
                    "c": 4,
                },
                "e": true,
            },
            "f": false,
        }));
    }
}
