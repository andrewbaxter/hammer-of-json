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

pub fn set(dest: &mut Supervalue, path: &DataPath, value: &Supervalue, missing_ok: bool) -> Result<(), String> {
    return at_path(
        //. .
        path,
        dest,
        || match missing_ok {
            true => AtPathEarlyRes::SetAndContinue,
            false => AtPathEarlyRes::Err,
        },
        || match missing_ok {
            true => AtPathEarlyRes::SetAndContinue,
            false => AtPathEarlyRes::Err,
        },
        |_, _| match missing_ok {
            true => AtPathEndRes::SetAndReturn(value.clone(), ()),
            false => AtPathEndRes::Err,
        },
        |parent, key| {
            parent.value.insert(key.to_string(), value.clone());
            return Ok(());
        },
        |root| {
            *root = value.clone();
            return Ok(());
        },
    );
}

#[cfg(test)]
mod test {
    use {
        super::set,
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
        set(
            &mut source,
            &DataPath(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            &Supervalue::from(json!("also_hello")),
            true,
        ).unwrap();
        assert_eq!(source, Supervalue::from(json!({
            "a": {
                "b": {
                    "c": "also_hello",
                    "d": "hello",
                },
                "e": true,
            },
            "f": false,
        })));
    }
}
