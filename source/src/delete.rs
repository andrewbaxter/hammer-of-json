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

pub fn delete(source: &mut Supervalue, path: &DataPath, missing_ok: bool) -> Result<(), String> {
    at_path(
        //. .
        &path,
        source,
        || match missing_ok {
            true => AtPathEarlyRes::Return(()),
            false => AtPathEarlyRes::Err,
        },
        || match missing_ok {
            true => AtPathEarlyRes::Return(()),
            false => AtPathEarlyRes::Err,
        },
        |_, _| match missing_ok {
            true => AtPathEndRes::Return(()),
            false => AtPathEndRes::Err,
        },
        |parent, key| {
            parent.value.remove(key);
            return Ok(());
        },
        |root| {
            *root = Supervalue::Null;
            return Ok(());
        },
    )?;
    return Ok(());
}

#[cfg(test)]
mod test {
    use {
        super::delete,
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
        delete(&mut source, &DataPath(vec!["a".to_string(), "b".to_string(), "c".to_string()]), true).unwrap();
        assert_eq!(source, Supervalue::from(json!({
            "a": {
                "b": {
                    "d": "hello",
                },
                "e": true,
            },
            "f": false,
        })));
    }
}
