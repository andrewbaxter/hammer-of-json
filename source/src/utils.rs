use {
    crate::{
        supervalue::{
            Supervalue,
            SupervalueMap,
            SupervalueVec,
        },
        supervalue_path::DataPath,
    },
    std::collections::hash_map::Entry,
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

pub enum AtPathResVec<T> {
    Return(T),
    Err,
}

pub fn at_path<
    T,
>(
    path: &DataPath,
    mut at: &mut Supervalue,
    mut handle_early_missing: impl FnMut() -> AtPathEarlyRes<T>,
    mut handle_early_missing_vec: impl FnMut() -> AtPathResVec<T>,
    mut handle_early_untraversible: impl FnMut() -> AtPathEarlyRes<T>,
    handle_end_missing: impl FnOnce(&mut SupervalueMap, &str) -> AtPathEndRes<T>,
    handle_end_found: impl FnOnce(&mut SupervalueMap, &str) -> Result<T, String>,
    handle_end_missing_vec: impl FnOnce(&mut SupervalueVec, usize) -> AtPathResVec<T>,
    handle_end_found_vec: impl FnOnce(&mut SupervalueVec, usize) -> Result<T, String>,
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
                Supervalue::Vec(_) => {
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
            match at {
                Supervalue::Map(map) => {
                    let seg = match seg {
                        serde_json::Value::String(s) => s,
                        _ => {
                            return Err(
                                format!(
                                    "At a map, but path contains non-string segment {} [{}]",
                                    depth,
                                    serde_json::to_string(seg).unwrap()
                                ),
                            );
                        },
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
                        let v = match map.value.entry(seg.clone()) {
                            Entry::Occupied(v) => v.into_mut(),
                            Entry::Vacant(en) => match handle_early_missing() {
                                AtPathEarlyRes::Return(v) => {
                                    return Ok(v);
                                },
                                AtPathEarlyRes::SetAndContinue => {
                                    en.insert(Supervalue::Map(Default::default()))
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
                            },
                        };
                        at = v;
                    }
                },
                Supervalue::Vec(ve) => {
                    let seg = match seg {
                        serde_json::Value::String(s) => match str::parse::<usize>(s) {
                            Ok(seg) => seg,
                            Err(e) => {
                                return Err(
                                    format!(
                                        "At an array, but path contains non-numberlike segment {} [{}]: {}",
                                        depth,
                                        serde_json::to_string(seg).unwrap(),
                                        e
                                    ),
                                );
                            },
                        },
                        serde_json::Value::Number(n) => {
                            let n = n.as_f64().unwrap();
                            if n < 0. {
                                return Err(
                                    format!("At an array, but path contains negative index {} [{}]", depth, n),
                                );
                            }
                            n as usize
                        },
                        _ => {
                            return Err(
                                format!(
                                    "At a map, but path contains non-string segment {} [{}]",
                                    depth,
                                    serde_json::to_string(seg).unwrap()
                                ),
                            );
                        },
                    };
                    if last {
                        if seg < ve.value.len() {
                            return handle_end_found_vec(ve, seg);
                        } else {
                            match handle_end_missing_vec(ve, seg) {
                                AtPathResVec::Return(v) => {
                                    return Ok(v);
                                },
                                AtPathResVec::Err => {
                                    return Err(
                                        format!(
                                            "Encountered array value at {:?} but the key [{:?}] is out of bounds",
                                            &path.0[0 .. depth],
                                            seg
                                        ),
                                    );
                                },
                            }
                        }
                    } else {
                        let Some(v) = ve.value.get_mut(seg) else {
                            match handle_early_missing_vec() {
                                AtPathResVec::Return(v) => {
                                    return Ok(v);
                                },
                                AtPathResVec::Err => {
                                    return Err(
                                        format!(
                                            "Encountered object value at {:?} but the key [{:?}] is missing",
                                            &path.0[0 .. depth],
                                            seg
                                        ),
                                    );
                                },
                            }
                        };
                        at = v;
                    }
                },
                _ => unreachable!(),
            };
        }
    }
    unreachable!();
}

pub enum SearchRes {
    Replace(Supervalue),
    Delete,
}

pub enum SearchKeyRes {
    Replace(String),
    Delete,
}

pub fn search(
    root: bool,
    at: &mut Supervalue,
    needle: &Supervalue,
    handle_end_found_in_obj: &mut impl FnMut() -> SearchRes,
    handle_end_found_in_key: &mut impl FnMut() -> SearchKeyRes,
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
                            &mut *handle_end_found_in_key,
                            &mut *handle_end_found_in_arr,
                            nil_handle_end,
                        );
                        i += 1;
                    }
                }
            },
            Supervalue::Map(map) => {
                'next_key: for mut k in map.value.keys().cloned().collect::<Vec<_>>() {
                    if let Supervalue::String(needle_str) = needle {
                        if &k == needle_str {
                            let v = map.value.remove(&k).unwrap();
                            match handle_end_found_in_key() {
                                SearchKeyRes::Replace(k2) => {
                                    map.value.insert(k2.clone(), v);
                                    k = k2;
                                },
                                SearchKeyRes::Delete => {
                                    continue 'next_key;
                                },
                            }
                        }
                    }
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
                            &mut *handle_end_found_in_key,
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
