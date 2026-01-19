use {
    crate::{
        merge::merge,
        set::set,
        supervalue::Supervalue,
        supervalue_path::DataPath,
        utils::{
            AtPathEarlyRes,
            AtPathEndRes,
            AtPathResVec,
            at_path,
        },
    },
    std::cell::RefCell,
};

pub fn keep(
    source: &mut Supervalue,
    out: &mut Option<Supervalue>,
    path: &DataPath,
    missing_ok: bool,
) -> Result<(), String> {
    let out = RefCell::new(out);
    at_path(
        //. .
        &path,
        source,
        || match missing_ok {
            true => AtPathEarlyRes::Return(()),
            false => AtPathEarlyRes::Err,
        },
        || match missing_ok {
            true => AtPathResVec::Return(()),
            false => AtPathResVec::Err,
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
            let mut temp = Supervalue::Map(Default::default());
            set(&mut temp, &path, &parent.value.remove(key).unwrap(), missing_ok)?;
            merge(out.borrow_mut().get_or_insert_with(|| Supervalue::Map(Default::default())), temp);
            return Ok(());
        },
        |_, _| match missing_ok {
            true => AtPathResVec::Return(()),
            false => AtPathResVec::Err,
        },
        |parent, key| {
            let mut temp = Supervalue::Map(Default::default());
            set(&mut temp, &path, &parent.value.remove(key), false)?;
            merge(out.borrow_mut().get_or_insert_with(|| Supervalue::Map(Default::default())), temp);
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
        let mut out = Some(Supervalue::from(json!({
            "a": {
                "e": true,
            }
        })));
        keep(&mut source, &mut out, &DataPath(vec![json!("a"), json!("b"), json!("c")]), true).unwrap();
        assert_eq!(out.unwrap(), Supervalue::from(json!({
            "a": {
                "b": {
                    "c": 4,
                },
                "e": true,
            }
        })));
    }
}
