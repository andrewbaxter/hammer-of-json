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

pub fn set(dest: &mut Supervalue, path: &DataPath, value: &Supervalue, missing_ok: bool) -> Result<(), String> {
    return at_path(
        //. .
        path,
        dest,
        || match missing_ok {
            true => AtPathEarlyRes::SetAndContinue,
            false => AtPathEarlyRes::Err,
        },
        || AtPathResVec::Err,
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
        |_, _| AtPathResVec::Err,
        |parent, key| {
            parent.value[key] = value.clone();
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
            &DataPath(vec![json!("a"), json!("b"), json!("c")]),
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

    #[test]
    fn set_in_array() {
        let mut source = Supervalue::from(json!({
            "a": {
                "b":[4, [5, "hello"],],
                "e": true,
            },
            "f": false,
        }));
        set(
            &mut source,
            &DataPath(vec![json!("a"), json!("b"), json!(1), json!(1)]),
            &Supervalue::from(json!("also_hello")),
            true,
        ).unwrap();
        assert_eq!(source, Supervalue::from(json!({
            "a": {
                "b":[4, [5, "also_hello"]],
                "e": true,
            },
            "f": false,
        })));
    }
}
