use {
    crate::{
        supervalue::Supervalue,
        utils::{
            search,
            SearchKeyRes,
            SearchRes,
        },
    },
    std::cell::Cell,
};

pub fn search_delete(source: &mut Supervalue, needle: &Supervalue) -> usize {
    let replacements = Cell::new(0);
    search(
        //. .
        true,
        source,
        needle,
        &mut || {
            replacements.set(replacements.get() + 1);
            return SearchRes::Delete;
        },
        &mut || {
            replacements.set(replacements.get() + 1);
            return SearchKeyRes::Delete;
        },
        &mut || {
            replacements.set(replacements.get() + 1);
            return SearchRes::Delete;
        },
        || {
            replacements.set(replacements.get() + 1);
            return SearchRes::Delete;
        },
    );
    return replacements.get();
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
