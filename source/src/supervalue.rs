use {
    aargvark::traits_impls::{
        AargvarkFile,
        AargvarkFromStr,
        AargvarkJson,
        AargvarkToml,
        AargvarkYaml,
    },
    flowcontrol::{
        exenum,
        shed,
    },
    jsonc_to_json::jsonc_to_json,
    samevariant::samevariant,
    std::collections::HashMap,
};

const YAML_TAG_TAG: &str = "tag";
const YAML_TAG_VALUE: &str = "tag";

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum SupervalueMapType {
    #[default]
    Normal,
    YamlTag,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SupervalueMap {
    pub type_: SupervalueMapType,
    pub value: HashMap<String, Supervalue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervalueVecType {
    Normal,
    YamlMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervalueVec {
    pub type_: SupervalueVecType,
    pub value: Vec<Supervalue>,
}

#[samevariant(SupervalueSamevariant)]
#[derive(Debug, Clone)]
pub enum Supervalue {
    Map(SupervalueMap),
    Vec(SupervalueVec),
    Null,
    Bool(bool),
    String(String),
    JsonNumber(serde_json::Number),
    YamlNumber(serde_yaml::Number),
    TomlDatetime(toml::value::Datetime),
}

impl PartialEq for Supervalue {
    fn eq(&self, other: &Self) -> bool {
        match SupervalueSamevariant::pairs(self, other) {
            SupervalueSamevariant::Null => return true,
            SupervalueSamevariant::Map(a, b) => return *a == *b,
            SupervalueSamevariant::Vec(a, b) => return *a == *b,
            SupervalueSamevariant::Bool(a, b) => return *a == *b,
            SupervalueSamevariant::String(a, b) => return *a == *b,
            SupervalueSamevariant::JsonNumber(a, b) => return *a == *b,
            SupervalueSamevariant::YamlNumber(a, b) => return *a == *b,
            SupervalueSamevariant::TomlDatetime(a, b) => return *a == *b,
            SupervalueSamevariant::Nonmatching(_, _) => return false,
        }
    }
}

impl Eq for Supervalue { }

impl From<serde_json::Value> for Supervalue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => return Self::Null,
            serde_json::Value::Bool(v) => return Self::Bool(v),
            serde_json::Value::Number(v) => return Self::JsonNumber(v),
            serde_json::Value::String(v) => return Self::String(v),
            serde_json::Value::Array(v) => return Self::Vec(SupervalueVec {
                type_: SupervalueVecType::Normal,
                value: v.into_iter().map(Supervalue::from).collect(),
            }),
            serde_json::Value::Object(v) => return Self::Map(SupervalueMap {
                type_: SupervalueMapType::Normal,
                value: v.into_iter().map(|(k, v)| (k, Supervalue::from(v))).collect(),
            }),
        }
    }
}

impl Into<serde_json::Value> for Supervalue {
    fn into(self) -> serde_json::Value {
        match self {
            Supervalue::Map(v) => return serde_json::Value::Object(
                v.value.into_iter().map(|(k, v)| (k, v.into())).collect(),
            ),
            Supervalue::Vec(v) => return serde_json::Value::Array(v.value.into_iter().map(|x| x.into()).collect()),
            Supervalue::Null => return serde_json::Value::Null,
            Supervalue::Bool(v) => return serde_json::Value::Bool(v),
            Supervalue::String(v) => return serde_json::Value::String(v),
            Supervalue::JsonNumber(v) => return serde_json::Value::Number(v),
            Supervalue::YamlNumber(v) => if v.is_f64() {
                if let Some(v) = serde_json::Number::from_f64(v.as_f64().unwrap().into()) {
                    return serde_json::Value::Number(v);
                } else {
                    return serde_json::Value::String(v.to_string());
                }
            } else if v.is_u64() {
                return serde_json::Value::Number(v.as_u64().unwrap().into());
            } else {
                return serde_json::Value::Number(v.as_i64().unwrap().into());
            },
            Supervalue::TomlDatetime(v) => return serde_json::Value::String(v.to_string()),
        }
    }
}

impl From<serde_yaml::Value> for Supervalue {
    fn from(value: serde_yaml::Value) -> Self {
        match value {
            serde_yaml::Value::Null => Supervalue::Null,
            serde_yaml::Value::Bool(v) => Supervalue::Bool(v),
            serde_yaml::Value::Number(v) => Supervalue::YamlNumber(v),
            serde_yaml::Value::String(v) => Supervalue::String(v),
            serde_yaml::Value::Sequence(v) => return Self::Vec(SupervalueVec {
                type_: SupervalueVecType::Normal,
                value: v.into_iter().map(Supervalue::from).collect(),
            }),
            serde_yaml::Value::Mapping(v) => {
                if v.keys().all(|k| exenum!(k, serde_yaml:: Value:: String(_) =>()).is_some()) {
                    return Self::Map(SupervalueMap {
                        type_: SupervalueMapType::Normal,
                        value: v
                            .into_iter()
                            .map(
                                |(k, v)| (
                                    exenum!(k, serde_yaml:: Value:: String(k) => k).unwrap(),
                                    Supervalue::from(v),
                                ),
                            )
                            .collect(),
                    });
                } else {
                    // Complex map, give up addressing and treat it as opaque array
                    return Self::Vec(SupervalueVec {
                        type_: SupervalueVecType::YamlMap,
                        value: v.into_iter().map(|(k, v)| Supervalue::Vec(SupervalueVec {
                            type_: SupervalueVecType::Normal,
                            value: vec![Supervalue::from(k), Supervalue::from(v)],
                        })).collect(),
                    });
                }
            },
            serde_yaml::Value::Tagged(v) => return Self::Map(SupervalueMap {
                type_: SupervalueMapType::YamlTag,
                value: {
                    let mut out = HashMap::new();
                    out.insert(YAML_TAG_TAG.to_string(), Supervalue::String(v.tag.to_string()));
                    out.insert(YAML_TAG_VALUE.to_string(), Supervalue::from(v.value));
                    out
                },
            }),
        }
    }
}

impl Into<serde_yaml::Value> for Supervalue {
    fn into(self) -> serde_yaml::Value {
        match self {
            Supervalue::Map(v) => {
                match v.type_ {
                    SupervalueMapType::Normal => {
                        // default result
                    },
                    SupervalueMapType::YamlTag => shed!{
                        let Some(Supervalue::String(tag)) = v.value.get(YAML_TAG_TAG) else {
                            break;
                        };
                        let Some(v) = v.value.get(YAML_TAG_VALUE) else {
                            break;
                        };
                        return serde_yaml::Value::Tagged(Box::new(serde_yaml::value::TaggedValue {
                            tag: serde_yaml::value::Tag::new(tag),
                            value: v.clone().into(),
                        }));
                    },
                }
                return serde_yaml::Value::Mapping(
                    v.value.into_iter().map(|(k, v)| (serde_yaml::Value::String(k), v.into())).collect(),
                );
            },
            Supervalue::Vec(v) => {
                match v.type_ {
                    SupervalueVecType::Normal => {
                        // default result
                    },
                    SupervalueVecType::YamlMap => shed!{
                        'bad_map _;
                        let mut out = serde_yaml::Mapping::with_capacity(v.value.len());
                        for kv in &v.value {
                            let Supervalue::Vec(kv) = kv else {
                                break 'bad_map;
                            };
                            let mut kv = kv.value.iter();
                            let k = kv.next().expect("Bad yaml complex map transfer, no key in array element");
                            let v = kv.next().expect("Bad yaml complex map transfer, no value in array element");
                            if kv.next().is_some() {
                                break 'bad_map;
                            }
                            out.insert(k.clone().into(), v.clone().into());
                        }
                        return serde_yaml::Value::Mapping(out);
                    },
                }
                return serde_yaml::Value::Sequence(v.value.into_iter().map(|x| x.into()).collect());
            },
            Supervalue::Null => return serde_yaml::Value::Null,
            Supervalue::Bool(v) => return serde_yaml::Value::Bool(v),
            Supervalue::String(v) => return serde_yaml::Value::String(v),
            Supervalue::YamlNumber(v) => return serde_yaml::Value::Number(v),
            Supervalue::JsonNumber(v) => if v.is_f64() {
                return serde_yaml::Value::Number(v.as_f64().unwrap().into());
            } else if v.is_u64() {
                return serde_yaml::Value::Number(v.as_u64().unwrap().into());
            } else {
                return serde_yaml::Value::Number(v.as_i64().unwrap().into());
            },
            Supervalue::TomlDatetime(v) => return serde_yaml::Value::String(v.to_string()),
        }
    }
}

impl From<toml::value::Value> for Supervalue {
    fn from(value: toml::value::Value) -> Self {
        match value {
            toml::Value::String(v) => return Self::String(v),
            toml::Value::Integer(v) => return Self::JsonNumber(serde_json::Number::from(v)),
            toml::Value::Float(v) => {
                if let Some(v) = serde_json::Number::from_f64(v) {
                    return Self::JsonNumber(v);
                } else {
                    return Self::String(v.to_string());
                }
            },
            toml::Value::Boolean(v) => return Self::Bool(v),
            toml::Value::Datetime(v) => return Self::TomlDatetime(v),
            toml::Value::Array(v) => return Self::Vec(SupervalueVec {
                type_: SupervalueVecType::Normal,
                value: v.into_iter().map(|v| v.into()).collect(),
            }),
            toml::Value::Table(v) => return Self::Map(SupervalueMap {
                type_: SupervalueMapType::Normal,
                value: v.into_iter().map(|(k, v)| (k, v.into())).collect(),
            }),
        }
    }
}

impl Into<toml::value::Value> for Supervalue {
    fn into(self) -> toml::value::Value {
        match self {
            Supervalue::Map(v) => return toml::value::Value::Table(
                v.value.into_iter().map(|(k, v)| (k, v.into())).collect(),
            ),
            Supervalue::Vec(v) => return toml::value::Value::Array(
                v.value.into_iter().map(|x| x.into()).collect(),
            ),
            Supervalue::Null => return toml::value::Value::String("null".to_string()),
            Supervalue::Bool(v) => return toml::value::Value::Boolean(v),
            Supervalue::String(v) => return toml::value::Value::String(v),
            Supervalue::JsonNumber(v) => if v.is_f64() {
                return toml::value::Value::Float(v.as_f64().unwrap());
            } else if v.is_i64() {
                return toml::value::Value::Integer(v.as_i64().unwrap());
            } else {
                return toml::value::Value::String(v.to_string());
            },
            Supervalue::YamlNumber(v) => if v.is_f64() {
                return toml::value::Value::Float(v.as_f64().unwrap());
            } else if v.is_i64() {
                return toml::value::Value::Integer(v.as_i64().unwrap());
            } else {
                return toml::value::Value::String(v.to_string());
            },
            Supervalue::TomlDatetime(v) => return toml::value::Value::Datetime(v),
        }
    }
}

pub enum AargSupervalueOriginalFormat {
    Json,
    Yaml,
    Toml,
}

pub struct AargSupervalue {
    pub original_format: AargSupervalueOriginalFormat,
    pub value: Supervalue,
    pub source: aargvark::traits_impls::Source,
}

impl AargvarkFromStr for AargSupervalue {
    fn from_str(s: &str) -> Result<Self, String> {
        if let Some(text) = s.strip_prefix("s:") {
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Json,
                value: Supervalue::String(text.into()),
                source: aargvark::traits_impls::Source::Stdin,
            });
        } else if let Some(path) = s.strip_prefix("fs:") {
            let t = AargvarkFile::from_str(path)?;
            let text = String::from_utf8(t.value).map_err(|e| format!("Invalid utf-8 in file [{}]: {}", path, e))?;
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Json,
                value: Supervalue::String(text),
                source: t.source,
            });
        } else if let Some(path) = s.strip_prefix("f:") {
            let t = AargvarkJson::<serde_json::Value>::from_str(&jsonc_to_json(path))?;
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Json,
                value: t.value.into(),
                source: t.source,
            });
        } else if let Some(path) = s.strip_prefix("fy:") {
            let t = AargvarkYaml::<serde_yaml::Value>::from_str(path)?;
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Yaml,
                value: t.value.into(),
                source: t.source,
            });
        } else if let Some(path) = s.strip_prefix("ft:") {
            let t = AargvarkToml::<toml::Value>::from_str(path)?;
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Toml,
                value: t.value.into(),
                source: t.source,
            });
        } else if let Some(v) = s.strip_prefix("y:") {
            let data =
                serde_yaml::from_str::<serde_yaml::Value>(
                    &v,
                ).map_err(|e| format!("Inline yaml [{}] is invalid: {}", v, e))?;
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Yaml,
                value: data.into(),
                source: aargvark::traits_impls::Source::Stdin,
            });
        } else if let Some(v) = s.strip_prefix("t:") {
            let data =
                toml::from_str::<toml::Value>(&v).map_err(|e| format!("Inline toml [{}] is invalid: {}", v, e))?;
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Toml,
                value: data.into(),
                source: aargvark::traits_impls::Source::Stdin,
            });
        } else {
            let data =
                serde_json::from_str::<serde_json::Value>(
                    &jsonc_to_json(s),
                ).map_err(|e| format!("Inline json [{}] is invalid: {}", s, e))?;
            return Ok(AargSupervalue {
                original_format: AargSupervalueOriginalFormat::Json,
                value: data.into(),
                source: aargvark::traits_impls::Source::Stdin,
            });
        }
    }

    fn build_help_pattern(_state: &mut aargvark::help::HelpState) -> aargvark::help::HelpPattern {
        return aargvark::help::HelpPattern(vec![aargvark::help::HelpPatternElement::Type("VALUE".to_string())]);
    }
}
