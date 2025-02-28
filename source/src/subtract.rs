fn recurse<
    'a,
>(
    errors: &mut Vec<String>,
    path: &mut Vec<&'a str>,
    source: &mut serde_json::Value,
    other: &'a serde_json::Value,
    missing_ok: bool,
) {
    match (source, other) {
        (serde_json::Value::Object(source), serde_json::Value::Object(other)) => {
            for (k, other_val) in other {
                path.push(k);
                if let Some(source_val) = source.get_mut(k) {
                    if source_val == other_val { } else {
                        recurse(errors, path, source_val, other_val, missing_ok);
                        path.pop();
                    }
                } else {
                    if missing_ok {
                        // nop
                    } else {
                        errors.push(
                            format!("Trying to subtract path [{:?}] but no value exists at that path", path),
                        );
                    }
                }
                source.remove(k);
            }
        },
        _ => {
            // no match, no subtraction
        },
    }
}

pub fn subtract(source: &mut serde_json::Value, other: &serde_json::Value, missing_ok: bool) -> Result<(), String> {
    if source == other {
        *source = serde_json::Value::Null;
    } else {
        let mut layer_errors = vec![];
        recurse(&mut layer_errors, &mut vec![], source, other, missing_ok);
        if !layer_errors.is_empty() {
            return Err(layer_errors.iter().map(|e| format!("- {}", e)).collect::<Vec<_>>().join("\n"));
        }
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
