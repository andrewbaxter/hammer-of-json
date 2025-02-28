This is a collection of tools for common json document manipulations.  It operates on entire documents, not streams, unlike `jq`.

Use it like:
```
$ joh set --in-place ./manifest.json global.meta f:./global_meta.json
$ joh search-set --in-place ./some.json '"_ARTIFACT_PLACEHOLDER"' '"/tmp/build/artifact.tar.gz"'
$ joh merge base.json layer1.json layer2.json > combined.json
$ joh validate-json-shchema combined.json
```

The command line help provides the best overview:

```
$ joh -h
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