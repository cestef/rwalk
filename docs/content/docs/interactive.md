+++
title = "Interactive"
weight = 20
+++

The interactive mode allows you to edit options on the fly and manipulate the results with the [`rhai`](https://rhai.rs) scripting language.

```bash, copy
rwalk -i # --interactive
```

## Commands

## Eval Mode

You can get into the eval mode by executing the `eval` (duh) command without any argument. 

```ansi
[034mrwalk>[0m eval
[034mrwalk[0m (eval)[034m>[0m 1 + 1  [2m// (´･Ω･｀) scary computation
2
```

If you just want to do a one-time exec, pass your expression directly to `eval`.

```ansi
[034mrwalk>[0m eval 1 / 0 [2m// will it explode ?
[031m⨯[0m Error: Division by zero: 1 / 0
```

Note that scope is maintained accross executions, so you can store stuff and use it later:

```ansi
[034mrwalk[0m (eval)[034m>[0m let a = "Hello"
[2m[^C
[034mrwalk>[0m eval
[034mrwalk[0m (eval)[034m>[0m print(`${a}, World!`)
Hello, World!
```