use {
    crate::{
        supervalue::Supervalue,
        utils::{
            search,
            SearchRes,
        },
    },
    std::cell::Cell,
};

pub fn search_set(source: &mut Supervalue, needle: &Supervalue, data: &Supervalue) -> usize {
    let replacements = Cell::new(0);
    search(
        //. .
        true,
        source,
        needle,
        &mut || {
            replacements.set(replacements.get() + 1);
            return SearchRes::Replace(data.clone());
        },
        &mut || {
            replacements.set(replacements.get() + 1);
            return SearchRes::Replace(data.clone());
        },
        || {
            replacements.set(replacements.get() + 1);
            return SearchRes::Replace(data.clone());
        },
    );
    return replacements.get();
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
