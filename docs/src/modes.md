# Modes

## Recursive

Recursive mode is the default mode of `rwalk`. It starts from a given path and checks each of its subdirectories.

Let's say you want to scan the `example.com` website with the following structure:

```
example.com
├── /about
│   ├── /team
│   │   └── /member1
│   └── /contact
└── /products
    ├── /product1
    └── /product2
```

Recursive mode will start from `example.com` and check:

- `example.com/about`
- `example.com/about/team`
- `example.com/about/team/member1`
- `example.com/about/contact`
- `example.com/products`
- `example.com/products/product1`
- `example.com/products/product2`
- ...


### Directory detection

`rwalk` will only recurse into directories. If a file is found, it will be ignored.
It determines if a path is a directory with a simple algorithm:

```python
if content is html:
    if content contains some of default html directories:
        return True
else if response is redirection and has a location header:
    if location header is equal to f"{url}/":
        return True
    else:
        return False
else if status is in 200-299 or 401-403:
    return True
else:
    return False
```

If this algorithm is not enough for your use case, you can implement your own directory detection function in the [`rhai`](https://rhai.rs) scripting language. See [Scripting](scripting.md) for more information.

        
## Classic

Classic mode allows for template-based fuzzing. You provide a list of patterns to check, and `rwalk` will replace each pattern with the words from the wordlists.

Each wordlist is identified by an optional key, which is used to reference it in the patterns.

For example, let's say you have two wordlists: `version.txt` and `endpoints.txt`. You want to find all paths that follow the pattern `/api/{version}/{endpoint}`. This can be achieved with the following command:

```ansi
rwalk example.com/api/[0;32mV[0m/[0;33mE[0m -w [0;32mversion.txt[0m:[0;32mV[0m -w [0;33mendpoints.txt[0m:[0;33mE[0m
```
<code class="language-ansi">[0;32mversion.txt[0m</code> is associated with the key <code class="language-ansi">[0;32mV[0m</code>, and <code class="language-ansi">[0;33mendpoints.txt[0m</code> is associated with the key <code class="language-ansi">[0;33mE[0m</code>.

<div class="note">
    The default key is <code>$</code>.
</div>

**Tip:** You could generate the `version.txt` directly with the following command:

```ansi
[0;34mseq 1 10[0m | [0;34mxargs -I{} echo v{}[0m | rwalk example.com/api/[0;32mV[0m/[0;33mE[0m -w [0;32m-[0m:[0;32mV[0m -w [0;33mendpoints.txt[0m:[0;33mE[0m
```

Using <code class="language-ansi">[0;32m-[0m</code> as the wordlist path will make `rwalk` read from the standard input.

These keys can also be used to identify the wordlists in the options. If you want to apply some filtering to only one of the wordlists, you can use the key to reference it.

```ansi
rwalk example.com/api/[0;32mV[0m/[0;33mE[0m -w [0;32mversion.txt[0m:[0;32mV[0m -w [0;33mendpoints.txt[0m:[0;33mE[0m [0;34m--wf "[[0;33mE[0m[0;34m]length:>5"[0m
```

Here, we are using <code class="language-ansi">[0;34m--wf[0m</code> (short for `--wordlist-filter`) to only keep the endpoints with at least 5 characters.

## Spider

Spider mode, aka crawling mode, starts from a given path and follows all links found until a certain depth. This is particularly useful for recon tasks to find all associated endpoints of a target. 

This is also the only mode that needs to be specified explicitly as it is impossible to detect automatically.

For example, to crawl the `cstef.dev` website with a depth of 4:

```ansi
rwalk cstef.dev [0;32m--mode spider [0;33m--depth 4 [0;34m--subdomains[0m
```

The <code class="language-ansi">[0;34m--subdomains[0m</code> flag will also include subdomains in the crawl.

```ansi
[0;32m✓[0m [0;2m200[0m / ([0;2mdir[0m)[0m
[0;2m├─ [0m🔍 cstef.dev[0m
[0;2m│  ├─ [0;32m✓[0m [0;2m200[0m / ([0;2mdir[0m)[0m
[0;2m│  ├─ [0;32m✓[0m [0;2m200[0m /android-chrome-512x512.png ([0;2mimage/png[0m)[0m
[0;2m│  ├─ [0;32m✓[0m [0;2m200[0m /favicon.ico ([0;2mimage/vnd.microsoft.icon[0m)[0m
[0;2m│  └─ [0;32m✓[0m [0;2m200[0m /assets ([0;2mapplication/javascript[0m)[0m
[0;2m│     ├─ [0;32m✓[0m [0;2m200[0m /index-d18fbe59.js ([0;2mapplication/javascript[0m)[0m
[0;2m│     └─ [0;32m✓[0m [0;2m200[0m /index-81baf222.css ([0;2mtext/css[0m)[0m
[0;2m├─ [0m🔍 blog.cstef.dev[0m
[0;2m│  ├─ [0;32m✓[0m [0;2m200[0m / ([0;2mdir[0m)[0m
[0;2m│  ├─ [0;32m✓[0m [0;2m200[0m /posts ([0;2mtext/html[0m)[0m
[0;2m│  │  ├─ [0;32m✓[0m [0;2m200[0m /solving-the-traefik-puzzle ([0;2mtext/html[0m)[0m
[0;2m│  │  └─ [0;32m✓[0m [0;2m200[0m /web-scanning-efficiently ([0;2mtext/html[0m)[0m
[0;2m│  └─ [0;32m✓[0m [0;2m200[0m /_next/static ([0;2mapplication/javascript[0m)[0m
[0;2m└─ [0m🔍 ctf.cstef.dev[0m
[0;2m   └─ [0;32m✓[0m [0;2m200[0m /api/login ([0;2mtext/html[0m)[0m
```