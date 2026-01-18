use {
    aargvark::{
        vark,
        Aargvark,
    },
    flowcontrol::superif,
    hammer_of_json::{
        delete::delete,
        get::get,
        intersect::intersect,
        keep::keep,
        merge::merge,
        search_delete::search_delete,
        search_set::search_set,
        set::set,
        subtract::subtract,
        supervalue::{
            AargSupervalue,
            AargSupervalueOriginalFormat,
            Supervalue,
        },
        supervalue_path::DataPath,
    },
    jsonschema::Validator,
    std::{
        env::current_dir,
        fs::write,
        path::PathBuf,
        process::exit,
    },
};

#[derive(Aargvark)]
struct GetCommand {
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Path to read from source
    path: DataPath,
}

#[derive(Aargvark)]
struct SetCommand {
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Where to place the data in `source`
    path: DataPath,
    /// Data to set in `source`
    data: AargSupervalue,
}

#[derive(Aargvark)]
struct DeleteCommand {
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Paths of data to remove from `source`
    paths: Vec<DataPath>,
}

#[derive(Aargvark)]
struct KeepCommand {
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Paths of data to keep in `source`
    paths: Vec<DataPath>,
}

#[derive(Aargvark)]
struct SearchSetCommand {
    /// Data to find in `source`
    needle: AargSupervalue,
    /// Data to replace `needle`
    data: AargSupervalue,
    /// Even if the needle isn't found don't exit with an error.
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
}

#[derive(Aargvark)]
struct SearchDeleteCommand {
    /// Data to delete from `source`
    needle: AargSupervalue,
    /// Even if the needle isn't found don't exit with an error.
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
}

#[derive(Aargvark)]
struct IntersectCommand {
    /// Data to intersect with `source`
    values: Vec<AargSupervalue>,
}

#[derive(Aargvark)]
struct SubtractCommand {
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Data to subtract from `source`
    values: Vec<AargSupervalue>,
}

#[derive(Aargvark)]
struct MergeCommand {
    /// Data to merge into `source`
    values: Vec<AargSupervalue>,
}

#[derive(Aargvark)]
struct ValidateJsonSchemaCommand {
    /// External schema to validate `source` against. Overrides `$schema` in `source`
    /// if present.
    external: Option<AargSupervalue>,
}

#[derive(Aargvark)]
#[vark(break_help)]
enum Command {
    /// Output just the subtree at a path
    Get(GetCommand),
    /// Replace/insert a subtree at a path
    Set(SetCommand),
    /// Remove the subtrees at paths
    Delete(DeleteCommand),
    /// Remove everything but the subtrees at paths
    Keep(KeepCommand),
    /// Search for matching values and replace them with a new value
    SearchSet(SearchSetCommand),
    /// Search for matching values and delete them
    SearchDelete(SearchDeleteCommand),
    /// Return the tree common to all trees. I.e. for `{"a": 1, "b": 2}` and
    /// `{"b": 2, "c": 3}` return `{"b": 2}`
    Intersect(IntersectCommand),
    /// Return the tree composed of elements not present in any of these other trees
    Subtract(SubtractCommand),
    /// Add the data in each file, sequentually. Objects fields are recursed, while all
    /// other values are replaced atomically
    Merge(MergeCommand),
    /// Validate a file against a schema, either internal (via a root `"$schema"` key)
    /// or external. Doesn't change the input, but exits with an error if validation
    /// fails.
    ValidateJsonSchema(ValidateJsonSchemaCommand),
}

#[derive(Aargvark, Default)]
enum Format {
    CompactJson,
    #[default]
    PrettyJson,
    Toml,
    Yaml,
}

/// This is a collection of tools for common json (and yaml, and toml) document
/// manipulations.
#[derive(Aargvark)]
struct Args {
    /// Output format, defaults to `pretty`
    #[vark(flag = "--format", flag = "-f")]
    format: Option<Format>,
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// If the result is a string value, output as an unquoted (non-json) string
    #[vark(flag = "--unquote", flag = "-u")]
    unquote: Option<()>,
    commands: Vec<Command>,
}

fn main1() -> Result<(), String> {
    let root_args = vark::<Args>();
    let mut at = root_args.source.value;
    for command in root_args.commands {
        match command {
            Command::Get(args) => {
                at = get(&mut at, &args.path, args.missing_ok.is_some())?.unwrap_or(Supervalue::Null);
            },
            Command::Set(args) => {
                set(&mut at, &args.path, &args.data.value, args.missing_ok.is_some())?;
            },
            Command::Delete(args) => {
                for path in args.paths {
                    delete(&mut at, &path, args.missing_ok.is_some())?;
                }
            },
            Command::Keep(args) => {
                let mut out = None;
                for path in args.paths {
                    keep(&mut at, &mut out, &path, args.missing_ok.is_some())?;
                }
                at = out.unwrap_or(Supervalue::Null);
            },
            Command::SearchSet(args) => {
                let change_count = search_set(&mut at, &args.needle.value, &args.data.value);
                if args.missing_ok.is_none() && change_count == 0 {
                    return Err(
                        format!(
                            "No changes made; couldn't find needle {}",
                            serde_json::to_string(
                                &<Supervalue as Into<serde_json::Value>>::into(args.needle.value),
                            ).unwrap()
                        ),
                    );
                }
            },
            Command::SearchDelete(args) => {
                let change_count = search_delete(&mut at, &args.needle.value);
                if args.missing_ok.is_none() && change_count == 0 {
                    return Err(
                        format!(
                            "No changes made; couldn't find needle {}",
                            serde_json::to_string(
                                &<Supervalue as Into<serde_json::Value>>::into(args.needle.value),
                            ).unwrap()
                        ),
                    );
                }
            },
            Command::Intersect(args) => {
                for other in args.values {
                    intersect(&mut at, &other.value);
                }
            },
            Command::Subtract(args) => {
                for (layer_index, arg) in args.values.iter().enumerate() {
                    if let Err(e) = subtract(&mut at, &arg.value, args.missing_ok.is_some()) {
                        return Err(format!("Failed to subtract layer {}:\n{}", layer_index, e));
                    }
                }
            },
            Command::Merge(args) => {
                for v in args.values {
                    merge(&mut at, v.value);
                }
            },
            Command::ValidateJsonSchema(args) => {
                let schema: serde_json::Value = if let Some(schema) = args.external {
                    schema.value.into()
                } else if let Some(Supervalue::String(addr)) =
                    get(&mut at, &DataPath(vec!["$schema".to_string()]), true)? {
                    if addr.starts_with("https://") || addr.starts_with("http:///") {
                        ureq::get(addr.as_str())
                            .call()
                            .map_err(|e| format!("Error sending request for external schema at [{}]: {}", addr, e))?
                            .body_mut()
                            .read_json()
                            .map_err(
                                |e| format!("Error reading JSON from external schema response at [{}]: {}", addr, e),
                            )?
                    } else {
                        let schema =
                            std::fs::read(
                                &addr,
                            ).map_err(|e| format!("Error loading schema [{}] from disk: {}", addr, e))?;
                        let schema =
                            serde_json::from_slice::<serde_json::Value>(
                                &schema,
                            ).map_err(|e| format!("Schema at [{}] is invalid JSON: {}", addr, e))?;
                        schema
                    }
                } else {
                    return Err(
                        format!("The data doesn't contain `$schema` and no external schema specified, cannot validate"),
                    );
                };

                struct MyRetriever {
                    working_directory: PathBuf,
                }

                impl jsonschema::Retrieve for MyRetriever {
                    fn retrieve(
                        &self,
                        uri: &jsonschema::Uri<String>,
                    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
                        match uri.scheme().as_str() {
                            "http" | "https" => {
                                return Ok(
                                    ureq::get(uri.as_str())
                                        .call()
                                        .map_err(
                                            |e| format!(
                                                "Error sending request for external resource at [{}]: {}",
                                                uri,
                                                e
                                            ),
                                        )?
                                        .body_mut()
                                        .read_json()
                                        .map_err(
                                            |e| format!(
                                                "Error reading JSON from external resource response at [{}]: {}",
                                                uri,
                                                e
                                            ),
                                        )?,
                                );
                            },
                            "json-schema" => {
                                let path = self.working_directory.join(uri.path().as_str().trim_start_matches("/"));
                                return Ok(
                                    serde_json::from_slice(
                                        &std::fs::read(
                                            &path,
                                        ).map_err(
                                            |e| format!("Error reading external resource at [{:?}]: {}", path, e),
                                        )?,
                                    )?,
                                );
                            },
                            "file" | "" => {
                                let path = self.working_directory.join(uri.path().as_str());
                                return Ok(
                                    serde_json::from_slice(
                                        &std::fs::read(
                                            &path,
                                        ).map_err(
                                            |e| format!("Error reading external resource at [{:?}]: {}", path, e),
                                        )?,
                                    )?,
                                );
                            },
                            scheme => {
                                return Err(
                                    std::io::Error::other(
                                        format!("Unimplemented resource url scheme: {}", scheme),
                                    ).into(),
                                );
                            },
                        }
                    }
                }

                let validator =
                    Validator::options()
                        .with_retriever(MyRetriever { working_directory: match &root_args.source.source {
                            aargvark::traits_impls::Source::Stdin => current_dir().map_err(
                                |e| format!(
                                    "No file source to root relative paths and couldn't determine working directory from executable working directory: {}",
                                    e
                                ),
                            )?,
                            aargvark::traits_impls::Source::File(v) => v
                                .canonicalize()
                                .map_err(
                                    |e| format!(
                                        "Error determining absolute path of source [{}]: {}",
                                        v.to_string_lossy(),
                                        e
                                    ),
                                )?
                                .parent()
                                .ok_or_else(
                                    || format!(
                                        "Could not determine parent directory of source file [{}]",
                                        v.to_string_lossy()
                                    ),
                                )?
                                .to_path_buf(),
                        } })
                        .build(&schema)
                        .map_err(|e| format!("Error interpreting JSON Schema as JSON Schema: {}", e))?;
                if let Err(e) = validator.validate(&at.clone().into()) {
                    eprintln!("{}", e);
                    exit(1);
                }
            },
        }
    }
    let v = superif!({
        if !root_args.unquote.is_some() {
            break 'quote;
        }
        let Supervalue::String(at) = at else {
            break 'quote;
        };
        at
    } 'quote {
        match root_args.format.unwrap_or(match root_args.source.original_format {
            AargSupervalueOriginalFormat::Json => Format::PrettyJson,
            AargSupervalueOriginalFormat::Yaml => Format::Yaml,
            AargSupervalueOriginalFormat::Toml => Format::Toml,
        }) {
            Format::CompactJson => {
                serde_json::to_string(&<Supervalue as Into::<serde_json::Value>>::into(at)).unwrap()
            },
            Format::PrettyJson => {
                serde_json::to_string_pretty(&<Supervalue as Into::<serde_json::Value>>::into(at)).unwrap()
            },
            Format::Toml => {
                toml::to_string_pretty(&<Supervalue as Into::<toml::Value>>::into(at)).unwrap()
            },
            Format::Yaml => {
                serde_yaml::to_string(&<Supervalue as Into::<serde_yaml::Value>>::into(at)).unwrap()
            },
        }
    });
    if root_args.in_place.is_some() {
        let aargvark::traits_impls::Source::File(p) = &root_args.source.source else {
            return Err("Requested in-place modification but source is not a filesystem path".to_string());
        };
        write(&p, v.as_bytes()).map_err(|e| format!("Error writing result to {:?}: {}", p, e))?;
    } else {
        print!("{}", v);
    }
    return Ok(());
}

fn main() {
    match main1() {
        Ok(_) => { },
        Err(e) => {
            eprintln!("Exiting with fatal error: {}", e);
            exit(1);
        },
    }
}
