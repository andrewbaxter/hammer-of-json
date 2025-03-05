use {
    crate::supervalue::Supervalue,
};

pub fn merge(dest: &mut Supervalue, other: Supervalue) {
    match (dest, other) {
        (Supervalue::Map(dest), Supervalue::Map(other)) => {
            for (k, other) in other.value {
                if let Some(dest) = dest.value.get_mut(&k) {
                    merge(dest, other);
                } else {
                    dest.value.insert(k, other);
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
                    "m": {
                        "a": 14,
                    }
                },
                "e": true,
            },
            "f": false,
        }));
        merge(&mut source, Supervalue::from(json!({
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
        })));
        assert_eq!(source, Supervalue::from(json!({
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
        })));
    }
}
