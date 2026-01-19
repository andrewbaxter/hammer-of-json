use {
    crate::{
        supervalue::Supervalue,
        supervalue_path::DataPath,
        utils::{
            AtPathEarlyRes,
            AtPathEndRes,
            AtPathResVec,
            at_path,
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
            true => AtPathResVec::Return(None),
            false => AtPathResVec::Err,
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
        |_, _| match missing_ok {
            true => AtPathResVec::Return(None),
            false => AtPathResVec::Err,
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
        let found = get(&mut source, &DataPath(vec![json!("a"), json!("b"), json!("c")]), false).unwrap().unwrap();
        assert_eq!(found, Supervalue::from(json!(4)));
    }

    #[test]
    fn get_in_array() {
        let mut source = Supervalue::from(json!({
            "a": {
                "b":[4, [5, "hello"]],
                "e": true,
            },
            "f": false,
        }));
        let found =
            get(&mut source, &DataPath(vec![json!("a"), json!("b"), json!(1), json!(0)]), false).unwrap().unwrap();
        assert_eq!(found, Supervalue::from(json!(5)));
    }

    #[test]
    fn get_in_array_with_str_index() {
        let mut source = Supervalue::from(json!({
            "a": {
                "b":[4, [5, "hello"]],
                "e": true,
            },
            "f": false,
        }));
        let found =
            get(&mut source, &DataPath(vec![json!("a"), json!("b"), json!("1"), json!("0")]), false)
                .unwrap()
                .unwrap();
        assert_eq!(found, Supervalue::from(json!(5)));
    }
}
