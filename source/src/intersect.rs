use {
    std::collections::HashSet,
};

fn recurse(
    source: &mut serde_json::Map<String, serde_json::Value>,
    other: &serde_json::Map<String, serde_json::Value>,
) {
    let mut source_keys = source.keys().cloned().collect::<HashSet<_>>();
    for (other_key, other_child) in other {
        let Some(source_child) = source.get_mut(other_key) else {
            continue;
        };
        source_keys.remove(other_key);
        if source_child == other_child {
            // nop
        } else if let (serde_json::Value::Object(source_child), serde_json::Value::Object(other_child)) =
            (source_child, other_child) {
            recurse(source_child, other_child);
        } else {
            source.remove(other_key);
        }
    }
    for k in source_keys {
        source.remove(&k);
    }
}

pub fn intersect(source: &mut serde_json::Value, other: &serde_json::Value) {
    if source == other {
        // nop
    } else if let (serde_json::Value::Object(source), serde_json::Value::Object(other)) = (&mut *source, other) {
        recurse(source, other);
    } else {
        *source = serde_json::Value::Null;
    }
}

#[cfg(test)]
mod test {
    use {
        super::intersect,
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
        intersect(&mut source, &json!({
            "a": {
                "e": true,
                "q": "gone"
            }
        }));
        assert_eq!(source, json!({
            "a": {
                "e": true,
            }
        }));
    }
}
