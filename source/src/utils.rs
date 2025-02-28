use {
    aargvark::traits_impls::{
        AargvarkFromStr,
        AargvarkJson,
    },
};

pub struct JsonPath(pub Vec<String>);

impl AargvarkFromStr for JsonPath {
    fn from_str(s: &str) -> Result<Self, String> {
        if s.starts_with("[") {
            return Ok(
                JsonPath(
                    serde_json::from_str(
                        s,
                    ).map_err(|e| format!("Error parsing path as JSON array of strings: {}", e))?,
                ),
            );
        } else {
            return Ok(
                JsonPath(s.trim_end_matches('.').split(".").map(|x| x.to_string()).collect::<Vec<_>>()),
            );
        }
    }

    fn build_help_pattern(_state: &mut aargvark::help::HelpState) -> aargvark::help::HelpPattern {
        return aargvark::help::HelpPattern(vec![aargvark::help::HelpPatternElement::Type(format!("PATH.TO.DATA"))]);
    }
}

pub struct JsonValue(pub AargvarkJson<serde_json::Value>);

impl AargvarkFromStr for JsonValue {
    fn from_str(s: &str) -> Result<Self, String> {
        if let Some(path) = s.strip_prefix("f:") {
            return Ok(JsonValue(AargvarkJson::<serde_json::Value>::from_str(path)?));
        } else {
            match serde_json::from_str(&s) {
                Ok(v) => {
                    return Ok(JsonValue(AargvarkJson {
                        value: v,
                        source: aargvark::traits_impls::Source::Stdin,
                    }));
                },
                Err(e) => {
                    return Err(e.to_string());
                },
            }
        }
    }

    fn build_help_pattern(state: &mut aargvark::help::HelpState) -> aargvark::help::HelpPattern {
        return aargvark::help::HelpPattern(
            vec![
                aargvark::help::HelpPatternElement::Variant(
                    vec![
                        AargvarkJson::<serde_json::Value>::build_help_pattern(state),
                        aargvark::help::HelpPattern(
                            vec![aargvark::help::HelpPatternElement::Type("JSON".to_string())],
                        )
                    ],
                )
            ],
        );
    }
}

pub enum AtPathEarlyRes<T> {
    Return(T),
    SetAndContinue,
    Err,
}

pub enum AtPathEndRes<T> {
    Return(T),
    SetAndReturn(serde_json::Value, T),
    Err,
}

pub fn at_path<
    T,
>(
    path: &JsonPath,
    mut at: &mut serde_json::Value,
    mut handle_early_missing: impl FnMut() -> AtPathEarlyRes<T>,
    mut handle_early_untraversible: impl FnMut() -> AtPathEarlyRes<T>,
    handle_end_missing: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>, &str) -> AtPathEndRes<T>,
    handle_end_found: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>, &str) -> Result<T, String>,
    handle_end_root: impl FnOnce(&mut serde_json::Value) -> Result<T, String>,
) -> Result<T, String> {
    if path.0.is_empty() {
        return handle_end_root(at);
    } else {
        for (depth, seg) in path.0.iter().enumerate() {
            let last = depth == path.0.len() - 1;
            match at {
                serde_json::Value::Object(_) => {
                    // nop
                },
                _ => {
                    match handle_early_untraversible() {
                        AtPathEarlyRes::Return(v) => {
                            return Ok(v);
                        },
                        AtPathEarlyRes::SetAndContinue => {
                            *at = serde_json::Value::Object(serde_json::Map::new());
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
                serde_json::Value::Object(map) => map,
                _ => unreachable!(),
            };
            if last {
                if map.contains_key(seg) {
                    return handle_end_found(map, seg);
                } else {
                    match handle_end_missing(map, seg) {
                        AtPathEndRes::Return(v) => {
                            return Ok(v);
                        },
                        AtPathEndRes::SetAndReturn(v, ret) => {
                            map.insert(seg.to_string(), v);
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
                if !map.contains_key(seg) {
                    match handle_early_missing() {
                        AtPathEarlyRes::Return(v) => {
                            return Ok(v);
                        },
                        AtPathEarlyRes::SetAndContinue => {
                            map.insert(seg.clone(), serde_json::Value::Object(serde_json::Map::new()));
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
                let Some(v) = map.get_mut(seg) else {
                    unreachable!();
                };
                at = v;
            }
        }
    }
    unreachable!();
}

pub enum SearchRes {
    Replace(serde_json::Value),
    Delete,
}

pub fn search(
    root: bool,
    at: &mut serde_json::Value,
    needle: &serde_json::Value,
    handle_end_found_obj: &mut impl FnMut() -> SearchRes,
    handle_end_found_arr: &mut impl FnMut() -> SearchRes,
    handle_end_root: impl FnOnce() -> SearchRes,
) {
    fn nil_handle_end() -> SearchRes {
        unreachable!();
    }

    if root && at == needle {
        match handle_end_root() {
            SearchRes::Replace(value) => *at = value,
            SearchRes::Delete => *at = serde_json::Value::Null,
        }
    } else {
        match &mut *at {
            serde_json::Value::Array(values) => {
                let mut i = 0;
                while i < values.len() {
                    if values[i] == *needle {
                        match handle_end_found_arr() {
                            SearchRes::Delete => {
                                values.remove(i);
                            },
                            SearchRes::Replace(v) => {
                                values[i] = v;
                                i += 1;
                            },
                        }
                    } else {
                        search(
                            false,
                            &mut values[i],
                            &*needle,
                            &mut *handle_end_found_obj,
                            &mut *handle_end_found_arr,
                            nil_handle_end,
                        );
                        i += 1;
                    }
                }
            },
            serde_json::Value::Object(map) => {
                for k in map.keys().cloned().collect::<Vec<_>>() {
                    if map[&k] == *needle {
                        match handle_end_found_obj() {
                            SearchRes::Replace(value) => {
                                map.insert(k, value);
                            },
                            SearchRes::Delete => {
                                map.remove(&k);
                            },
                        }
                    } else {
                        search(
                            false,
                            &mut map[&k],
                            &*needle,
                            &mut *handle_end_found_obj,
                            &mut *handle_end_found_arr,
                            nil_handle_end,
                        );
                    }
                }
            },
            _ => { },
        }
    }
}
