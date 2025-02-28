use {
    crate::utils::{
        at_path,
        AtPathEarlyRes,
        JsonPath,
    },
};

/// Can only error if `!missing_ok`.
pub fn get(
    root: &mut serde_json::Value,
    path: &JsonPath,
    missing_ok: bool,
) -> Result<Option<serde_json::Value>, String> {
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
            true => AtPathEarlyRes::Return(None),
            false => AtPathEarlyRes::Err,
        },
        |parent, key| {
            return Ok(Some(parent.get(key).unwrap().clone()));
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
        crate::utils::JsonPath,
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
        let found =
            get(&mut source, &JsonPath(vec!["a".to_string(), "b".to_string(), "c".to_string()]), true)
                .unwrap()
                .unwrap();
        assert_eq!(found, json!(4));
    }
}
