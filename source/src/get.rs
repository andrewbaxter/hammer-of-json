use {
    crate::{
        supervalue::Supervalue,
        supervalue_path::DataPath,
        utils::{
            at_path,
            AtPathEarlyRes,
            AtPathEndRes,
        },
    },
};

/// Can only error if `!missing_ok`.
pub fn get(root: &mut Supervalue, path: &DataPath, missing_ok: bool) -> Result<Option<Supervalue>, String> {
    return Ok(at_path(
        //. .
        path,
        root,
        || match missing_ok {
            true => AtPathEarlyRes::Return(None),
            false => AtPathEarlyRes::Err,
        },
        || match missing_ok {
            true => AtPathEarlyRes::Return(None),
            false => AtPathEarlyRes::Err,
        },
        |_, _| match missing_ok {
            true => AtPathEndRes::Return(None),
            false => AtPathEndRes::Err,
        },
        |parent, key| {
            return Ok(Some(parent.value.get(key).unwrap().clone()));
        },
        |v| {
            return Ok(Some(v.clone()));
        },
    )?);
}

#[cfg(test)]
mod test {
    use {
        super::get,
        crate::{
            supervalue::Supervalue,
            supervalue_path::DataPath,
        },
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
        let found =
            get(&mut source, &DataPath(vec!["a".to_string(), "b".to_string(), "c".to_string()]), true)
                .unwrap()
                .unwrap();
        assert_eq!(found, Supervalue::from(json!(4)));
    }
}
