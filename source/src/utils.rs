use {
    crate::{
        supervalue::{
            Supervalue,
            SupervalueMap,
        },
        supervalue_path::DataPath,
    },
};

pub enum AtPathEarlyRes<T> {
    Return(T),
    SetAndContinue,
    Err,
}

pub enum AtPathEndRes<T> {
    Return(T),
    SetAndReturn(Supervalue, T),
    Err,
}

pub fn at_path<
    T,
>(
    path: &DataPath,
    mut at: &mut Supervalue,
    mut handle_early_missing: impl FnMut() -> AtPathEarlyRes<T>,
    mut handle_early_untraversible: impl FnMut() -> AtPathEarlyRes<T>,
    handle_end_missing: impl FnOnce(&mut SupervalueMap, &str) -> AtPathEndRes<T>,
    handle_end_found: impl FnOnce(&mut SupervalueMap, &str) -> Result<T, String>,
    handle_end_root: impl FnOnce(&mut Supervalue) -> Result<T, String>,
) -> Result<T, String> {
    if path.0.is_empty() {
        return handle_end_root(at);
    } else {
        for (depth, seg) in path.0.iter().enumerate() {
            let last = depth == path.0.len() - 1;
            match at {
                Supervalue::Map(_) => {
                    // nop
                },
                _ => {
                    match handle_early_untraversible() {
                        AtPathEarlyRes::Return(v) => {
                            return Ok(v);
                        },
                        AtPathEarlyRes::SetAndContinue => {
                            *at = Supervalue::Map(Default::default());
                        },
                        AtPathEarlyRes::Err => {
                            return Err(
                                format!(
                                    "Encountered primitive value at {:?}, before reaching end of path",
                                    &path.0[0 ..= depth]
                                ),
                            );
                        },
                    }
                },
            }
            let map = match at {
                Supervalue::Map(map) => map,
                _ => unreachable!(),
            };
            if last {
                if map.value.contains_key(seg) {
                    return handle_end_found(map, seg);
                } else {
                    match handle_end_missing(map, seg) {
                        AtPathEndRes::Return(v) => {
                            return Ok(v);
                        },
                        AtPathEndRes::SetAndReturn(v, ret) => {
                            map.value.insert(seg.to_string(), v);
                            return Ok(ret);
                        },
                        AtPathEndRes::Err => {
                            return Err(
                                format!(
                                    "Encountered object value at {:?} but the key [{:?}] is missing",
                                    &path.0[0 .. depth],
                                    seg
                                ),
                            );
                        },
                    }
                }
            } else {
                if !map.value.contains_key(seg) {
                    match handle_early_missing() {
                        AtPathEarlyRes::Return(v) => {
                            return Ok(v);
                        },
                        AtPathEarlyRes::SetAndContinue => {
                            map.value.insert(seg.clone(), Supervalue::Map(Default::default()));
                        },
                        AtPathEarlyRes::Err => {
                            return Err(
                                format!(
                                    "Encountered object value at {:?} but the key [{:?}] is missing",
                                    &path.0[0 .. depth],
                                    seg
                                ),
                            );
                        },
                    }
                }
                let Some(v) = map.value.get_mut(seg) else {
                    unreachable!();
                };
                at = v;
            }
        }
    }
    unreachable!();
}

pub enum SearchRes {
    Replace(Supervalue),
    Delete,
}

pub fn search(
    root: bool,
    at: &mut Supervalue,
    needle: &Supervalue,
    handle_end_found_in_obj: &mut impl FnMut() -> SearchRes,
    handle_end_found_in_arr: &mut impl FnMut() -> SearchRes,
    handle_end_found_at_root: impl FnOnce() -> SearchRes,
) {
    fn nil_handle_end() -> SearchRes {
        unreachable!();
    }

    if root && at == needle {
        match handle_end_found_at_root() {
            SearchRes::Replace(value) => *at = value,
            SearchRes::Delete => *at = Supervalue::Null,
        }
    } else {
        match &mut *at {
            Supervalue::Vec(values) => {
                let mut i = 0;
                while i < values.value.len() {
                    if values.value[i] == *needle {
                        match handle_end_found_in_arr() {
                            SearchRes::Delete => {
                                values.value.remove(i);
                            },
                            SearchRes::Replace(v) => {
                                values.value[i] = v;
                                i += 1;
                            },
                        }
                    } else {
                        search(
                            false,
                            &mut values.value[i],
                            &*needle,
                            &mut *handle_end_found_in_obj,
                            &mut *handle_end_found_in_arr,
                            nil_handle_end,
                        );
                        i += 1;
                    }
                }
            },
            Supervalue::Map(map) => {
                for k in map.value.keys().cloned().collect::<Vec<_>>() {
                    if map.value[&k] == *needle {
                        match handle_end_found_in_obj() {
                            SearchRes::Replace(value) => {
                                map.value.insert(k, value);
                            },
                            SearchRes::Delete => {
                                map.value.remove(&k);
                            },
                        }
                    } else {
                        search(
                            false,
                            &mut map.value.get_mut(&k).unwrap(),
                            &*needle,
                            &mut *handle_end_found_in_obj,
                            &mut *handle_end_found_in_arr,
                            nil_handle_end,
                        );
                    }
                }
            },
            _ => { },
        }
    }
}
