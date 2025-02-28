pub fn merge(dest: &mut serde_json::Value, other: serde_json::Value) {
    match (dest, other) {
        (serde_json::Value::Object(dest), serde_json::Value::Object(other)) => {
            for (k, other) in other {
                if let Some(dest) = dest.get_mut(&k) {
                    merge(dest, other);
                } else {
                    dest.insert(k, other);
                }
            }
        },
        (dest, other) => {
            *dest = other;
        },
    }
}

#[cfg(test)]
mod test {
    use {
        super::merge,
        serde_json::json,
    };

    #[test]
    fn base() {
        let mut source = json!({
            "a": {
                "b": {
                    "c": 4,
                    "d": "hello",
                    "m": {
                        "a": 14,
                    }
                },
                "e": true,
            },
            "f": false,
        });
        merge(&mut source, json!({
            "b": 44,
            "a": {
                "b": {
                    "c": 4,
                    "m": 3,
                },
                "e": {
                    "q": 12,
                }
            }
        }));
        assert_eq!(source, json!({
            "b": 44,
            "a": {
                "b": {
                    "c": 4,
                    "d": "hello",
                    "m": 3,
                },
                "e": {
                    "q": 12,
                }
            },
            "f": false,
        }));
    }
}
