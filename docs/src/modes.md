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

## Spider