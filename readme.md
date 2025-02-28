This is a collection of tools for common json document manipulations.  It operates on entire documents, not streams, unlike `jq`.

Use it like:
```
$ hoj set --in-place ./manifest.json global.meta f:./global_meta.json
$ hoj search-set --in-place ./some.json '"_ARTIFACT_PLACEHOLDER"' '"/tmp/build/artifact.tar.gz"'
$ hoj merge base.json layer1.json layer2.json > combined.json
$ hoj validate-json-shchema combined.json
```

The command line help provides the best overview:

```
$ hoj -h
Usage: hoj SOURCE COMMAND [ ...FLAGS]

    SOURCE: <PATH> | - | <JSON>    Source JSON file
    COMMAND: COMMAND
    [--in-place]                   Modify source in-place
    [-i]                           (synonym for `--in-place`)
    [--unquote]                    If the result is a string value, output as an unquoted (non-json) string
    [-u]                           (synonym for `--unquote`)
    [--format FORMAT]              Output format, defaults to `pretty`
    [-f FORMAT]                    (synonym for `--format`)
    [--missing-ok]                 If a value referred to by a path, values to replace, or data to subtract is missing, don't 
                                   abort (treat as ok).
    [-m]                           (synonym for `--missing-ok`)

COMMAND: array | get | set | delete | keep | search-set | search-delete | intersect | subtract | merge | validate-json-schema

    array ...                      Create an array from arguments.  Arguments are parsed as JSON, if that fails they're turned
                                   into JSON strings. To make values into strings explicitly, add quotes. (ex, in bash: `'"123"'`)
    get ...                        Get the subtree at a path, outputting the subtree
    set ...                        Replace/insert a subtree at a path, outputting the modified data
    delete ...                     Remove the subtrees at paths, returning the remaining data
    keep ...                       Remove the subtrees at paths, returning just the removed data
    search-set ...                 Search for matching values and replace them with a new value
    search-delete ...              Search for matching values and delete them
    intersect ...                  Return the tree common to all trees. I.e. for `{"a": 1, "b": 2}` and `{"b": 2, "c": 3}`
                                   return `{"b": 2}`
    subtract ...                   Return the tree that's not present in any of these files
    merge ...                      Add the data in each file, sequentually. Objects are recursed and merged per-key, while all other 
                                   values are replaced
    validate-json-schema ...       Validate a file against a schema, either internal (via a root `"$schema"` key) or external

FORMAT: compact | pretty

    compact
    pretty
```

# Conventions

## Paths

A number of commands take json paths.

A json path can be (and is automatically deduced to be):

- A number of `.` delimited segments, like `a.b.c`. These don't take escapes, so only simple strings (no `.`) segments are allowed.

- A json array of strings: `["a", "b", "c"]`

This path addresses the data at the key `a` of the root object, `b` of that object, and then the value at key `c` in that object.

## Values

A number of the commands take values.  Values can either be inline json, a path to a json file prefixed with `f:`, or `-` to indicate to read json data from stdin.

Only one value can come from stdin, if a command takes multiple arguments.

Note that your shell probably interprets quotes, so in order to write a json string you may need to double-quote: `'"my text"'`

## Arrays

For most operations, arrays are treated as "primitives" and no deep manipulation happens.

The relation and ordering of elements in an array is often significant, and you need per-application logic for determining the right way to modify the array.  The implementation of things like merges are ambiguous and have many potential exclusive implementations.

Generally speaking, in any JSON document the array as it is is typically in a valid configuration, so either entirely keeping or entirely deleting the array is the safest.

Where the implementation is unambiguous these tools attempt to support arrays.

### Workarounds

If you find a situation where you need to manipulate arrays, try:

1. Working with upstream to find a way to turn the array into an object instead.  Do this for situations where you want a "set" too, by creating an object with your unique values mapped to boolean values and have the application ignore any values set to `false`.
2. Using objects in your own data and implementing a post-processing step to convert the object back into an array.