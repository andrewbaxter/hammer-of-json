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
struct ArrayCommand {
    elements: Vec<String>,
}

#[derive(Aargvark)]
struct GetCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// If the result is a string value, output as an unquoted (non-json) string
    #[vark(flag = "--unquote", flag = "-u")]
    unquote: Option<()>,
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Path to read from source
    path: DataPath,
}

#[derive(Aargvark)]
struct SetCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// If the result is a string value, output as an unquoted (non-json) string
    #[vark(flag = "--unquote", flag = "-u")]
    unquote: Option<()>,
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
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Paths of data to remove from `source`
    paths: Vec<DataPath>,
}

#[derive(Aargvark)]
struct KeepCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Paths of data to keep in `source`
    paths: Vec<DataPath>,
}

#[derive(Aargvark)]
struct SearchSetCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// Data to find in `source`
    needle: AargSupervalue,
    /// Data to replace `needle`
    data: AargSupervalue,
}

#[derive(Aargvark)]
struct SearchDeleteCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// Data to delete from `source`
    needle: AargSupervalue,
}

#[derive(Aargvark)]
struct IntersectCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// Data to intersect with `source`
    values: Vec<AargSupervalue>,
}

#[derive(Aargvark)]
struct SubtractCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    /// Data to subtract from `source`
    values: Vec<AargSupervalue>,
}

#[derive(Aargvark)]
struct MergeCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// Data to merge into `source`
    values: Vec<AargSupervalue>,
}

#[derive(Aargvark)]
struct ValidateJsonSchemaCommand {
    /// Source JSON file
    source: AargSupervalue,
    /// External schema to validate `source` against. Overrides `$schema` in `source`
    /// if present.
    external: Option<AargSupervalue>,
}

#[derive(Aargvark)]
#[vark(break_help)]
enum Command {
    /// Create an array from arguments.  Arguments are parsed as JSON, if that fails
    /// they're turned into JSON strings. To make values into strings explicitly, add
    /// quotes. (ex, in bash: `'"123"'`)
    Array(ArrayCommand),
    /// Get the subtree at a path, outputting the subtree
    Get(GetCommand),
    /// Replace/insert a subtree at a path, outputting the modified data
    Set(SetCommand),
    /// Remove the subtrees at paths, returning the remaining data
    Delete(DeleteCommand),
    /// Remove the subtrees at paths, returning just the removed data
    Keep(KeepCommand),
    /// Search for matching values and replace them with a new value
    SearchSet(SearchSetCommand),
    /// Search for matching values and delete them
    SearchDelete(SearchDeleteCommand),
    /// Return the tree common to all trees. I.e. for `{"a": 1, "b": 2}` and
    /// `{"b": 2, "c": 3}` return `{"b": 2}`
    Intersect(IntersectCommand),
    /// Return the tree that's not present in any of these files
    Subtract(SubtractCommand),
    /// Add the data in each file, sequentually. Objects are recursed and merged
    /// per-key, while all other values are replaced
    Merge(MergeCommand),
    /// Validate a file against a schema, either internal (via a root `"$schema"` key)
    /// or external
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
    command: Command,
}

fn output(
    v: Supervalue,
    source: aargvark::traits_impls::Source,
    original_format: AargSupervalueOriginalFormat,
    unquote: bool,
    in_place: bool,
    format: Option<Format>,
) -> Result<(), String> {
    let v = superif!({
        if !unquote {
            break 'quote;
        }
        let Supervalue::String(v) = v else {
            break 'quote;
        };
        v
    } 'quote {
        match format.unwrap_or(match original_format {
            AargSupervalueOriginalFormat::Json => Format::PrettyJson,
            AargSupervalueOriginalFormat::Yaml => Format::Yaml,
            AargSupervalueOriginalFormat::Toml => Format::Toml,
        }) {
            Format::CompactJson => {
                serde_json::to_string(&<Supervalue as Into::<serde_json::Value>>::into(v)).unwrap()
            },
            Format::PrettyJson => {
                serde_json::to_string_pretty(&<Supervalue as Into::<serde_json::Value>>::into(v)).unwrap()
            },
            Format::Toml => {
                toml::to_string_pretty(&<Supervalue as Into::<toml::Value>>::into(v)).unwrap()
            },
            Format::Yaml => {
                serde_yaml::to_string(&<Supervalue as Into::<serde_yaml::Value>>::into(v)).unwrap()
            },
        }
    });
    if in_place {
        let aargvark::traits_impls::Source::File(p) = &source else {
            return Err("Requested in-place modification but source is not a filesystem path".to_string());
        };
        write(&p, v.as_bytes()).map_err(|e| format!("Error writing result to {:?}: {}", p, e))?;
    } else {
        print!("{}", v);
    }
    return Ok(());
}

fn main1() -> Result<(), String> {
    let root_args = vark::<Args>();
    match root_args.command {
        Command::Array(args) => {
            let mut out = vec![];
            for arg in args.elements {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&arg) {
                    out.push(v.into());
                } else {
                    out.push(Supervalue::String(arg));
                }
            }
            let out = match root_args.format.unwrap_or_default() {
                Format::CompactJson => serde_json::to_string(&serde_json::Value::from(out)).unwrap(),
                Format::PrettyJson => serde_json::to_string_pretty(&serde_json::Value::from(out)).unwrap(),
                Format::Toml => toml::to_string_pretty(&toml::Value::from(out)).unwrap(),
                Format::Yaml => serde_yaml::to_string(&serde_yaml::Value::from(out)).unwrap(),
            };
            print!("{}", out);
        },
        Command::Get(args) => {
            let mut source = args.source.value;
            let at = get(&mut source, &args.path, args.missing_ok.is_some())?.unwrap_or(Supervalue::Null);
            output(
                at,
                args.source.source,
                args.source.original_format,
                args.unquote.is_some(),
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::Set(args) => {
            let mut out = args.source.value;
            set(&mut out, &args.path, &args.data.value, args.missing_ok.is_some())?;
            output(
                out,
                args.source.source,
                args.source.original_format,
                args.unquote.is_some(),
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::Delete(args) => {
            let mut out = args.source.value;
            for path in args.paths {
                delete(&mut out, &path, args.missing_ok.is_some())?;
            }
            output(
                out,
                args.source.source,
                args.source.original_format,
                false,
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::Keep(args) => {
            let mut source = args.source.value;
            let mut out = None;
            for path in args.paths {
                keep(&mut source, &mut out, &path, args.missing_ok.is_some())?;
            }
            output(
                out.unwrap_or(Supervalue::Null),
                args.source.source,
                args.source.original_format,
                false,
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::SearchSet(args) => {
            let mut out = args.source.value;
            search_set(&mut out, &args.needle.value, &args.data.value);
            output(
                out,
                args.source.source,
                args.source.original_format,
                false,
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::SearchDelete(args) => {
            let mut out = args.source.value;
            search_delete(&mut out, &args.needle.value);
            output(
                out,
                args.source.source,
                args.source.original_format,
                false,
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::Intersect(args) => {
            let mut out = args.source.value;
            for other in args.values {
                intersect(&mut out, &other.value);
            }
            output(
                out,
                args.source.source,
                args.source.original_format,
                false,
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::Subtract(args) => {
            let mut out = args.source.value;
            for (layer_index, arg) in args.values.iter().enumerate() {
                if let Err(e) = subtract(&mut out, &arg.value, args.missing_ok.is_some()) {
                    return Err(format!("Failed to subtract layer {}:\n{}", layer_index, e));
                }
            }
            output(
                out,
                args.source.source,
                args.source.original_format,
                false,
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::Merge(args) => {
            let mut out = args.source.value;
            for v in args.values {
                merge(&mut out, v.value);
            }
            output(
                out,
                args.source.source,
                args.source.original_format,
                false,
                args.in_place.is_some(),
                root_args.format,
            )?;
        },
        Command::ValidateJsonSchema(args) => {
            let mut source = args.source.value;
            let schema: serde_json::Value = if let Some(schema) = args.external {
                schema.value.into()
            } else if let Some(Supervalue::String(addr)) =
                get(&mut source, &DataPath(vec!["$schema".to_string()]), true)? {
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
                                        |e| format!("Error sending request for external resource at [{}]: {}", uri, e),
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
                        "file" | "" => {
                            let path = self.working_directory.join(uri.path().as_str());
                            return Ok(
                                serde_json::from_slice(
                                    &std::fs::read(
                                        &path,
                                    ).map_err(|e| format!("Error reading external resource at [{:?}]: {}", path, e))?,
                                )?,
                            );
                        },
                        scheme => {
                            return Err(
                                std::io::Error::other(format!("Unimplemented resource url scheme: {}", scheme)).into(),
                            );
                        },
                    }
                }
            }

            let validator =
                Validator::options().with_retriever(MyRetriever { working_directory: match &args.source.source {
                    aargvark::traits_impls::Source::Stdin => current_dir().map_err(
                        |e| format!(
                            "No file source to root relative paths and couldn't determine working directory from executable working directory: {}",
                            e
                        ),
                    )?,
                    aargvark::traits_impls::Source::File(v) => v.clone(),
                } }).build(&schema).map_err(|e| format!("Error interpreting JSON Schema as JSON Schema: {}", e))?;
            if let Err(e) = validator.validate(&source.into()) {
                eprintln!("{}", e);
                exit(1);
            }
        },
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
