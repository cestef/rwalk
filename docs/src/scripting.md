# Scripting

`rwalk` supports scripting with the [`rhai`](https://rhai.rs) scripting language. You can use scripting to implement custom directory detection functions, custom filters, or any other custom logic you need.

## Directory detection

You can provide a custom directory detection script to `rwalk` in the form of a `rhai` script. The script must return a boolean value indicating whether a path is a directory or not.

It can be passed to `rwalk` using the `--directory-script` (`--dr`) option.

Here is an example of a custom directory detection script:

```rs
// custom_directory_detection.rhai

if response.headers.get("content-type") == "text/html" {
    return true;
}
return false;
```

You have access to the following variables:

- `response` [ScriptingResponse](https://github.com/cestef/rwalk/tree/main/src/runner/filters.rs#L422): The response data (compact version of reqwest's `Response`)
- `opts` [Opts](https://github.com/cestef/rwalk/tree/main/src/cli/opts.rs#L22): The options passed to `rwalk`.

## Filters

Custom filters can also be implemented using `rhai`. Filters must return a boolean value indicating whether a response should be kept or not.

To pass a custom filter script to `rwalk`, use it's path with the <code class="language-ansi">[0;34m--filter</code> (`-f`) option.

```ansi
rwalk example.com [0;34m--filter [0;33mcustom_filter.rhai[0m:[0;32margument[0m
```

Here is an example of a custom filter script:

```rs
// custom_filter.rhai
if response.body.contains(input) {
    return true;
}
return false;
```


You have access to the following variables:

- `response` [ScriptingResponse](https://github.com/cestef/rwalk/tree/main/src/runner/filters.rs#L422): The response data (compact version of reqwest's `Response`)
- `opts` [Opts](https://github.com/cestef/rwalk/tree/main/src/cli/opts.rs#L22): The options passed to `rwalk`.
- `input` [String](https://rhai.rs/book/language/strings-chars.html#strings-and-characters): The argument passed to the filter.