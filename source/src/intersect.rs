use {
    crate::supervalue::{
        Supervalue,
        SupervalueMap,
    },
    std::collections::{
        HashSet,
    },
};

fn recurse(source: &mut SupervalueMap, other: &SupervalueMap) {
    let mut source_keys = source.value.keys().cloned().collect::<HashSet<_>>();
    for (other_key, other_child) in &other.value {
        let Some(source_child) = source.value.get_mut(other_key) else {
            continue;
        };
        source_keys.remove(other_key);
        if source_child == other_child {
            // nop
        } else if let (Supervalue::Map(source_child), Supervalue::Map(other_child)) = (source_child, other_child) {
            recurse(source_child, &other_child);
        } else {
            source.value.remove(other_key);
        }
    }
    for k in source_keys {
        source.value.remove(&k);
    }
}

pub fn intersect(source: &mut Supervalue, other: &Supervalue) {
    if source == other {
        // nop
    } else if let (Supervalue::Map(source), Supervalue::Map(other)) = (&mut *source, other) {
        recurse(source, &other);
    } else {
        *source = Supervalue::Null;
    }
}

#[cfg(test)]
mod test {
    use {
        super::intersect,
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
        intersect(&mut source, &Supervalue::from(json!({
            "a": {
                "e": true,
                "q": "gone"
            }
        })));
        assert_eq!(source, Supervalue::from(json!({
            "a": {
                "e": true,
            }
        })));
    }
}
