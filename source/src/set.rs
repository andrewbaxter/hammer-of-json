use {
    crate::utils::{
        at_path,
        AtPathEarlyRes,
        AtPathEndRes,
        JsonPath,
    },
};

pub fn set(
    dest: &mut serde_json::Value,
    path: &JsonPath,
    value: &serde_json::Value,
    missing_ok: bool,
) -> Result<(), String> {
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
            parent.insert(key.to_string(), value.clone());
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
        set(
            &mut source,
            &JsonPath(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            &json!("also_hello"),
            true,
        ).unwrap();
        assert_eq!(source, json!({
            "a": {
                "b": {
                    "c": "also_hello",
                    "d": "hello",
                },
                "e": true,
            },
            "f": false,
        }));
    }
}
