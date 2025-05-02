This is a collection of tools for common json document manipulations. It operates on entire documents, not streams, unlike `jq`.

While it's named hammer of _json_, it is also a hammer of _yaml_ and a cudgel of _toml_.

The command line help provides the best overview:

```
$ hoj -h
Usage: hoj SOURCE COMMANDS [ ...FLAGS]

    This is a collection of tools for common json (and yaml, and toml) document manipulations.

 SOURCE: <s:STRING> | <fs:PATH> | <f:PATH> | <fy:PATH> | <ft:PTAH> | <y:YAML> | <t:TOML> | <JSON>
        Source JSON file
    COMMANDS: COMMAND[ ...]
    [--format FORMAT]              Output format, defaults to `pretty`
    [-f FORMAT]                    (synonym for `--format`)
    [--in-place]                   Modify source in-place
    [-i]                           (synonym for `--in-place`)
    [--unquote]                    If the result is a string value, output as anunquoted (non-json) string
    [-u]                           (synonym for `--unquote`)

COMMAND: get | set | delete | keep | search-set | search-delete | intersect | subtract | merge | validate-json-schema

    get ...                        Get the subtree at a path, outputting the subtree
    set ...                        Replace/insert a subtree at a path, outputting the modified data
    delete ...                     Remove the subtrees at paths, returning the remaining data
    keep ...                       Remove the subtrees at paths, returning just the removed data
    search-set ...                 Search for matching values and replace them with a new value
    search-delete ...              Search for matching values and delete them
    intersect ...                  Return the tree common to all trees. I.e. for `{"a": 1, "b": 2}` and
                                     `{"b": 2, "c": 3}` return `{"b": 2}`
    subtract ...                   Return the tree that's not present in any of these files
    merge ...                      Add the data in each file, sequentually. Objects are recursed and merged
                                     per-key, while all other values are replaced
    validate-json-schema ...       Validate a file against a schema, either internal (via a root `"$schema"`
                                     key) or external

FORMAT: compact-json | pretty-json | toml | yaml

    compact-json
    pretty-json
    toml
    yaml
```

The subcommands are pipelined, with the output of the previous command being the input of the next command, so you can do things like:

```
hoj f:fdap.json \
    search-set s:__SET_ADMIN_TOKEN "s:$ADMIN_TOKEN" \
    search-set s:__SET_XYZ_PASSWORD "s:$XYZ_PASSWORD" \
    search-set s:__SET_QUERY_ALBUMS "$query_albums_json" \
    search-set s:__SET_QUERY_ALBUMS_TRACKS "$query_albums_tracks_json" \
    search-set s:__SET_QUERY_NOTES "$query_notes_json" \
    validate-json-schema
```

In-place modification and referencing external files in `validate-json-schema` are done relative to the source file path and pipelining preserves this context through all the commands.

# Conventions

There are two main data types used in arguments: _paths_ and _values_.

## Paths

A path can be:

- A number of `.` prefixed segments, like `.a.b.c`. These don't take escapes, so only simple string (no internal `.`'s) segments are allowed.

- A json array of strings: `["a", "b", "c"]`

These are unambiguous - `hoj` automatically detects which type of path it is.

This path addresses corresponds to the value at key `a`, then the value at key `b` within that, and then the value at key `c` within that.

## Values

A value can be:

- Inline json

- A string, prefixed by `s:`

- Inline yaml, prefixed by `y:`

- Inline toml, prefixed by `t:`

- A path prefixed by `fs:` (like `fs:./a.txt`) referring to the contents of a plain text file to be treated as a string

- A path prefixed by `f:` (like `f:./a.json`) referring to the contents of a json file

- A path prefixed by `fy:` referring to the contents of a yaml file

- A path prefixed by `ft:` referring to the contents of a toml file

Paths can also be `-` to read from stdin.

Note that your shell probably interprets quotes and other symbols, so depending on the value you may need to extra-quote. For instance, for a json string you may need to write: `'"my text"'`

## Arrays

For most operations, arrays are treated as "primitives" and no deep manipulation happens.

There are a couple reasons for this:

- The relation and ordering of elements in an array is often significant

- The implementation of things like merges are ambiguous and have many potential incompatible implementations

- How do you represent a partial array? Unlike maps you may need to have the right array dimensions, but putting any element in the empty spaces could end up replacing legitimate values during the operation

Generally speaking, in any JSON document the array as it is is typically in a valid configuration, so either entirely keeping or entirely deleting the array is often the safest choice.

Where the implementation is unambiguous these tools attempt to support arrays.

If you find a situation where you need to manipulate arrays, try:

1. Working with upstream to find a way to turn the array into an object instead. Do this for situations where you want a "set" too, by creating an object with your unique values mapped to boolean values and have the application ignore any values set to `false`.

2. Using objects in your own data and implementing a post-processing step to convert the object back into an array.

3. Duplicating the array - for instance, if you want to merge document A into B and replace one element of an array in B, instead replace the whole array in B.

## Taml, Yoml

These are supported, but there are some limitations inherent and otherwise:

- Yaml dates aren't recognized and converted (ex: to/from toml dates)

- Yaml complex map keys can't be referenced by paths

- Conversions between formats may cause values to lose format-specific meanings or change format
