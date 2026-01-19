# Hammer of JSON

<img src="hammer.svg" align="left">

Hammer of JSON (`hoj`) is a collection of tools for manipulating JSON documents. It makes it easy to do things like merge files and template JSON.

While it's named hammer of _JSON_, it is also a hammel of _YAML_ and a cudgel of _TOML_ (and [JSONC](https://jsonc.org/)).

The command line help provides the best overview:

```
$ hoj -h
Usage: hoj SOURCE COMMANDS [ ...FLAGS]

    This is a collection of tools for common json (and yaml, and toml) document
    manipulations.

    SOURCE: <VALUE>          Source JSON file
    COMMANDS: COMMAND[ ...]
    [--format FORMAT]        Output format, defaults to `pretty`
    [-f FORMAT]              (synonym for `--format`)
    [--in-place]             Modify source in-place
    [-i]                     (synonym for `--in-place`)
    [--unquote]              If the result is a string value, output as an
                             unquoted (non-json) string
    [-u]                     (synonym for `--unquote`)

COMMAND: get | set | delete | keep | search-set | search-delete | intersect | su
btract | merge | validate-json-schema

    get ...                   Output just the subtree at a path.
    set ...                   Replace/insert a subtree at a path.
    delete ...                Remove the subtrees at paths. This will remove
                              the key from the parent map or element from the
                              parent array shifting later elements earlier.
    keep ...                  Remove everything but the subtrees at paths.
    search-set ...            Search for matching values and replace them with
                              a new value. If the value is found in a map, the
                              key that holds it will be removed with the value.
                              If it's found in an array, the array element will
                              be removed shifting later elements earlier.
    search-delete ...         Search for matching values and delete them.
    intersect ...             Return the tree common to all trees. I.e. for
                              `{"a": 1, "b": 2}` and `{"b": 2, "c": 3}` return
                              `{"b": 2}`.
    subtract ...              Return the tree composed of elements not present
                              in any of these other trees.
    merge ...                 Add the data in each file, sequentually. Objects
                              fields are recursed, while all other values are
                              replaced atomically.
    validate-json-schema ...  Validate a file against a schema, either internal
                              (via a root `"$schema"` key) or external. Doesn't
                              change the input, but exits with an error if
                              validation fails.

FORMAT: compact-json | pretty-json | toml | yaml

    compact-json
    pretty-json
    toml
    yaml
```

You can use multiple commands, forming a pipeline, where the output of the previous operation becomes input of the next. You can do things like:

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

You can also do

```
hoj f:fdap.json
```

to just format it.

# Conventions

There are two main data types used in arguments: _paths_ and _values_.

## Paths (to values in the data)

A path can be:

- A number of `.` prefixed segments, like `.a.b.c`. These don't take escapes, so only simple string (no internal `.`'s) segments are allowed.

- A JSON array of strings and numbers: `["a", "b", "c"]`

These are unambiguous - `hoj` automatically detects which type of path it is by the first letter.

This path addresses corresponds to the value at key `a`, then the value at key `b` within that, and then the value at key `c` within that. Numbers are used for array indices (i.e. `.0` would return the 0th element of the root array).

## Values

A value can be:

- Inline JSON, like `'"a string"'` (quoted for shell) or `6`

- A path prefixed by `f:` (like `f:./a.json`) referring to the contents of a JSON file

- A path prefixed by `fjc:` (like `fjc:./a.jsonc`) referring to the contents of a ([JSONC](https://jsonc.org/)) file

- A string, prefixed by `s:` (avoiding the need for nested quotes)

- A path prefixed by `fs:` (like `fs:./a.txt`) referring to the contents of a plain text file to be treated as a string

- Inline YAML, prefixed by `y:`

- A path prefixed by `fy:` referring to the contents of a YAML file

- Inline toml, prefixed by `t:`

- A path prefixed by `ft:` referring to the contents of a toml file

Paths can also be `-` to read from stdin.

Note that your shell probably interprets quotes and other symbols, so depending on the value you may need to extra-quote. For instance, for a JSON string you may need to write: `'"my text"'`

## Arrays

For merges, intersections, and some other operations, arrays are treated as "primitives" and no deep manipulation happens.

There are a couple reasons for this:

- The relation and ordering of elements in an array is often significant

- The implementation of things like merges are ambiguous and have many potential incompatible implementations

- There's no unambiguous way to represent a partial array (e.g. for merges, or the result of intersections), without significantly complicating other syntax.

Generally speaking, when you have two arrays, both are valid, so choosing one or the other atomically is often safer than trying to intelligently merge them.

Where the implementation is unambiguous these tools do attempt to support arrays.

If you find a situation where you need to manipulate arrays, try:

1. Working with upstream to find a way to turn the array into an object instead. Do this for situations where you want a "set" too, by creating an object with your unique values mapped to boolean values and have the application ignore any values set to `false`.

2. Using objects in your own data and implementing a post-processing step to convert the object back into an array.

3. Duplicating the array - for instance, if you want to merge document A into B and replace one element of an array in B, copy the whole array in B into A and modify it there.

## Taml, Yoml

These are supported, but there are some limitations inherent and otherwise:

- Yaml dates aren't recognized and converted (ex: to/from toml dates)

- Paths can't point to Yaml complex map keys

- Conversions between formats may cause values to lose format-specific meanings or change format
