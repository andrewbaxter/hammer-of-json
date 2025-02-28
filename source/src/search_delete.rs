use {
    crate::utils::{
        search,
        SearchRes,
    },
};

pub fn search_delete(source: &mut serde_json::Value, needle: &serde_json::Value) {
    search(true, source, needle, &mut || SearchRes::Delete, &mut || SearchRes::Delete, || SearchRes::Delete);
}

#[cfg(test)]
mod test {
    use {
        super::search_delete,
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
        search_delete(&mut source, &json!("hello"));
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
