use {
    crate::supervalue::{
        Supervalue,
        SupervalueMap,
    },
};

fn recurse<
    'a,
>(
    errors: &mut Vec<String>,
    path: &mut Vec<&'a str>,
    source: &mut SupervalueMap,
    other: &'a SupervalueMap,
    missing_ok: bool,
) {
    for (k, other_val) in &other.value {
        path.push(k);
        if let Some(source_val) = source.value.get_mut(k) {
            if source_val == other_val {
                source.value.remove(k);
            } else if let (Supervalue::Map(source), Supervalue::Map(other)) = (source_val, other_val) {
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

pub fn subtract(source: &mut Supervalue, other: &Supervalue, missing_ok: bool) -> Result<(), String> {
    let mut layer_errors = vec![];
    if source == other {
        *source = Supervalue::Null;
    } else if let (Supervalue::Map(source), Supervalue::Map(other)) = (source, other) {
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
        crate::supervalue::Supervalue,
        serde_json::json,
    };

    #[test]
    fn base() {
        let mut source = Supervalue::from(json!({
            "a": {
                "b": {
                    "c": 4,
                    "d": "hello",
                },
                "e": true,
            },
            "f": false,
        }));
        subtract(&mut source, &Supervalue::from(json!({
            "a": {
                "b": {
                    "d": "hello",
                }
            }
        })), true).unwrap();
        assert_eq!(source, Supervalue::from(json!({
            "a": {
                "b": {
                    "c": 4,
                },
                "e": true,
            },
            "f": false,
        })));
    }
}
