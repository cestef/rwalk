# Scripting

`rwalk` supports scripting with the [`rhai`](https://rhai.rs) scripting language. You can use scripting to implement custom directory detection functions, custom filters, or any other custom logic you need.

## Directory detection

You can provide a custom directory detection script to `rwalk` in the form of a `rhai` script. The script must return a boolean value indicating whether a path is a directory or not.

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