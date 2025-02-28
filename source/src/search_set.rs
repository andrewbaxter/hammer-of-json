use {
    crate::utils::{
        search,
        SearchRes,
    },
};

pub fn search_set(source: &mut serde_json::Value, needle: &serde_json::Value, data: &serde_json::Value) {
    search(
        true,
        source,
        needle,
        &mut || SearchRes::Replace(data.clone()),
        &mut || SearchRes::Replace(data.clone()),
        || SearchRes::Replace(data.clone()),
    );
}

#[cfg(test)]
mod test {
    use {
        super::search_set,
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
        search_set(&mut source, &json!("hello"), &json!("goodbye"));
        assert_eq!(source, json!({
            "a": {
                "b": {
                    "c": 4,
                    "d": "goodbye",
                },
                "e": true,
            },
            "f": false,
        }));
    }
}
