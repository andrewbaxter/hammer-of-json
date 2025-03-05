use {
    crate::{
        supervalue::Supervalue,
        utils::{
            search,
            SearchRes,
        },
    },
};

pub fn search_set(source: &mut Supervalue, needle: &Supervalue, data: &Supervalue) {
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
        search_set(&mut source, &Supervalue::from(json!("hello")), &Supervalue::from(json!("goodbye")));
        assert_eq!(source, Supervalue::from(json!({
            "a": {
                "b": {
                    "c": 4,
                    "d": "goodbye",
                },
                "e": true,
            },
            "f": false,
        })));
    }
}
