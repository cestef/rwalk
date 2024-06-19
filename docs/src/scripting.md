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
  
| Variable   | Description                                                 | Type                                                                                                                                                          |
| ---------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `response` | The response data (compact version of reqwest's `Response`) | <a href="https://docs.rs/rwalk/latest/rwalk/runner/filters/struct.ScriptingResponse.html"><code class="language-ansi">[0;32mScriptingResponse[0m</code></a> |
| `opts`     | The options passed to `rwalk`                               | <a href="https://docs.rs/rwalk/latest/rwalk/cli/opts/struct.Opts.html"><code class="language-ansi">[0;32mOpts[0m</code></a>                                 |

```ansi

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

| Variable   | Description                                                 | Type                                                                                                                                                          |
| ---------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `response` | The response data (compact version of reqwest's `Response`) | <a href="https://docs.rs/rwalk/latest/rwalk/runner/filters/struct.ScriptingResponse.html"><code class="language-ansi">[0;32mScriptingResponse[0m</code></a> |
| `opts`     | The options passed to `rwalk`                               | <a href="https://docs.rs/rwalk/latest/rwalk/cli/opts/struct.Opts.html"><code class="language-ansi">[0;32mOpts[0m</code></a>                                 |
| `input`    | The argument passed to the filter                           | <a href="https://rhai.rs/book/language/strings-chars.html#strings-and-characters"><code class="language-ansi">[0;33mString[0m</code></a>                    |
## Interactive mode

Scripting is available through the `eval` command in the interactive mode (`--interactive`, `-i`). You can use this to easily analyze the reponses or run custom logic. 

Here are the available variables:

| Variable | Description                                     | Type                                                                                                                                                 |
| -------- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tree`   | If the scan is complete, the tree of found URLs | <a href="https://docs.rs/rwalk/latest/rwalk/utils/tree/index.html"><code class="language-ansi">[0;32smTreeNode[0m\<[0;34mTreeData[0m></code></a> |
| `opts`   | The options passed to `rwalk`                   | <a href="https://docs.rs/rwalk/latest/rwalk/cli/opts/struct.Opts.html"><code class="language-ansi">[0;32mOpts[0m</code></a>                        |
