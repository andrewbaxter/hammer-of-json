use {
    crate::utils::{
        at_path,
        AtPathEarlyRes,
        AtPathEndRes,
        JsonPath,
    },
};

pub fn delete(source: &mut serde_json::Value, path: &JsonPath, missing_ok: bool) -> Result<(), String> {
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
            parent.remove(key);
            return Ok(());
        },
        |root| {
            *root = serde_json::Value::Null;
            return Ok(());
        },
    )?;
    return Ok(());
}

#[cfg(test)]
mod test {
    use {
        super::delete,
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
        delete(&mut source, &JsonPath(vec!["a".to_string(), "b".to_string(), "c".to_string()]), true).unwrap();
        assert_eq!(source, json!({
            "a": {
                "b": {
                    "d": "hello",
                },
                "e": true,
            },
            "f": false,
        }));
    }
}
