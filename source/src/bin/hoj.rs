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
        utils::{
            JsonPath,
            JsonValue,
        },
    },
    std::{
        fs::write,
        process::exit,
    },
};

#[derive(Aargvark)]
struct SetCommand {
    path: JsonPath,
    data: JsonValue,
}

#[derive(Aargvark)]
struct SearchSetCommand {
    needle: JsonValue,
    data: JsonValue,
}

#[derive(Aargvark)]
struct SearchDeleteCommand {
    needle: JsonValue,
}

#[derive(Aargvark)]
struct ValidateJsonSchemaCommand {
    external: Option<JsonValue>,
}

#[derive(Aargvark)]
enum Command {
    /// Create an array from arguments.  Arguments are parsed as JSON, if that fails
    /// they're turned into JSON strings. To make values into strings explicitly, add
    /// quotes. (ex, in bash: `'"123"'`)
    Array(Vec<String>),
    /// Get the subtree at a path, outputting the subtree
    Get(JsonPath),
    /// Replace/insert a subtree at a path, outputting the modified data
    Set(SetCommand),
    /// Remove the subtrees at paths, returning the remaining data
    Delete(Vec<JsonPath>),
    /// Remove the subtrees at paths, returning just the removed data
    Keep(Vec<JsonPath>),
    /// Search for matching values and replace them with a new value
    SearchSet(SearchSetCommand),
    /// Search for matching values and delete them
    SearchDelete(SearchDeleteCommand),
    /// Return the tree common to all trees. I.e. for `{"a": 1, "b": 2}` and
    /// `{"b": 2, "c": 3}` return `{"b": 2}`
    Intersect(Vec<JsonValue>),
    /// Return the tree that's not present in any of these files
    Subtract(Vec<JsonValue>),
    /// Add the data in each file, sequentually. Objects are recursed and merged
    /// per-key, while all other values are replaced
    Merge(Vec<JsonValue>),
    /// Validate a file against a schema, either internal (via a root `"$schema"` key)
    /// or external
    ValidateJsonSchema(ValidateJsonSchemaCommand),
}

#[derive(Aargvark)]
enum Format {
    Compact,
    Pretty,
}

#[derive(Aargvark)]
#[vark(break_help)]
struct Args {
    /// Source JSON file
    source: JsonValue,
    /// Modify source in-place
    #[vark(flag = "--in-place", flag = "-i")]
    in_place: Option<()>,
    /// If the result is a string value, output as an unquoted (non-json) string
    #[vark(flag = "--unquote", flag = "-u")]
    unquote: Option<()>,
    /// Output format, defaults to `pretty`
    #[vark(flag = "--format", flag = "-f")]
    format: Option<Format>,
    /// If a value referred to by a path, values to replace, or data to subtract is
    /// missing, don't abort (treat as ok).
    #[vark(flag = "--missing-ok", flag = "-m")]
    missing_ok: Option<()>,
    command: Command,
}

fn main1() -> Result<(), String> {
    let root_args = vark::<Args>();
    let missing_ok = root_args.missing_ok.is_some();
    let mut source = root_args.source.0.value;
    let output = |v: serde_json::Value| -> Result<(), String> {
        let v = superif!({
            if !root_args.unquote.is_some() {
                break 'quote;
            }
            let serde_json::Value::String(v) = v else {
                break 'quote;
            };
            v
        } 'quote {
            match root_args.format.unwrap_or(Format::Pretty) {
                Format::Compact => {
                    serde_json::to_string(&v).unwrap()
                },
                Format::Pretty => {
                    serde_json::to_string_pretty(&v).unwrap()
                },
            }
        });
        if root_args.in_place.is_some() {
            let aargvark::traits_impls::Source::File(p) = root_args.source.0.source else {
                return Err("Requested in-place modification but source is not a filesystem path".to_string());
            };
            write(&p, v.as_bytes()).map_err(|e| format!("Error writing result to {:?}: {}", p, e))?;
        } else {
            print!("{}", v);
        }
        return Ok(());
    };
    match root_args.command {
        Command::Array(args) => {
            let mut out = vec![];
            for arg in args {
                if let Ok(v) = serde_json::from_str(&arg) {
                    out.push(v);
                } else {
                    out.push(serde_json::Value::String(arg));
                }
            }
            output(serde_json::Value::Array(out))?;
        },
        Command::Get(args) => {
            let at = get(&mut source, &args, missing_ok)?.unwrap_or(serde_json::Value::Null);
            output(at)?;
        },
        Command::Set(args) => {
            set(&mut source, &args.path, &args.data.0.value, missing_ok)?;
            output(source)?;
        },
        Command::Delete(args) => {
            for path in args {
                delete(&mut source, &path, missing_ok)?;
            }
            output(source)?;
        },
        Command::Keep(args) => {
            let mut out = None;
            for path in args {
                keep(&mut source, &mut out, &path, missing_ok)?;
            }
            output(out.unwrap_or(serde_json::Value::Null))?;
        },
        Command::SearchSet(args) => {
            search_set(&mut source, &args.needle.0.value, &args.data.0.value);
            output(source)?;
        },
        Command::SearchDelete(args) => {
            search_delete(&mut source, &args.needle.0.value);
            output(source)?;
        },
        Command::Intersect(args) => {
            for other in args {
                intersect(&mut source, &other.0.value);
            }
            output(source)?;
        },
        Command::Subtract(args) => {
            for (layer_index, arg) in args.iter().enumerate() {
                if let Err(e) = subtract(&mut source, &arg.0.value, root_args.missing_ok.is_some()) {
                    return Err(format!("Failed to subtract layer {}:\n{}", layer_index, e));
                }
            }
            output(source)?;
        },
        Command::Merge(args) => {
            let mut source = source;
            for v in args {
                merge(&mut source, v.0.value);
            }
            output(source)?;
        },
        Command::ValidateJsonSchema(args) => {
            let schema = if let Some(schema) = args.external {
                schema.0.value
            } else if let Some(serde_json::Value::String(addr)) =
                get(&mut source, &JsonPath(vec!["$schema".to_string()]), true)? {
                if addr.starts_with("https://") {
                    todo!();
                } else if addr.starts_with("http:///") {
                    todo!();
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
            if let Err(e) = jsonschema::validate(&schema, &source) {
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
