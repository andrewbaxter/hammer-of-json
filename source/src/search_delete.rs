use {
    crate::{
        supervalue::Supervalue,
        utils::{
            search,
            SearchRes,
        },
    },
};

pub fn search_delete(source: &mut Supervalue, needle: &Supervalue) {
    search(true, source, needle, &mut || SearchRes::Delete, &mut || SearchRes::Delete, || SearchRes::Delete);
}

#[cfg(test)]
mod test {
    use {
        super::search_delete,
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
        search_delete(&mut source, &Supervalue::from(json!("hello")));
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
