use {
    crate::{
        set::set,
        merge::merge,
        utils::{
            at_path,
            AtPathEarlyRes,
            JsonPath,
        },
    },
};

pub fn keep(
    source: &mut serde_json::Value,
    out: &mut Option<serde_json::Value>,
    path: &JsonPath,
    missing_ok: bool,
) -> Result<(), String> {
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
            true => AtPathEarlyRes::Return(()),
            false => AtPathEarlyRes::Err,
        },
        |parent, key| {
            let mut temp = serde_json::Value::Object(serde_json::Map::new());
            set(&mut temp, &path, &parent.remove(key).unwrap(), missing_ok)?;
            merge(out.get_or_insert_with(|| serde_json::Value::Object(Default::default())), temp);
            return Ok(());
        },
        |_root| {
            // nop
            return Ok(());
        },
    )?;
    return Ok(());
}

#[cfg(test)]
mod test {
    use {
        super::keep,
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
        let mut out = Some(json!({
            "a": {
                "e": true,
            }
        }));
        keep(
            &mut source,
            &mut out,
            &JsonPath(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            true,
        ).unwrap();
        assert_eq!(out.unwrap(), json!({
            "a": {
                "b": {
                    "c": 4,
                },
                "e": true,
            }
        }));
    }
}
