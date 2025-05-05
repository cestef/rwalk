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
[034mrwalk[0m [2m(eval)[034m>[0m 1 + 1  [2m// \(˚☐˚”)/ scary computation
2
```

If you just want to do a one-time exec, pass your expression directly to `eval`.

```ansi
[034mrwalk>[0m eval 1 / 0 [2m// will it explode ?
[031m⨯[0m Error: Division by zero: 1 / 0
```

Note that scope is maintained accross executions, so you can store stuff and use it later:

```ansi
[034mrwalk[0m [2m(eval)[034m>[0m let a = "Hello"
[2m[^C
[034mrwalk>[0m eval
[034mrwalk[0m [2m(eval)[034m>[0m print(`${a}, World!`)
Hello, World!
```

To evaluate a script, just prepend `@` before your file name:

```ansi
[034mrwalk>[0m eval @path/to/script.rhai
Hello, World!
[034mrwalk[0m [2m(eval)[034m>[0m @path/to/script.rhai
Hello, World!
```

## Manipulating Options

Let's start out by listing the current options:

```ansi
[034mrwalk>[0m list
(B[0;1mbell(B[0m (B[0;2m············(B[0m = [31mfalse
(B[0;1mconfig(B[0m (B[0;2m··········(B[0m = (B[0;2mnull
(B[0;1mdepth(B[0m (B[0;2m···········(B[0m = [93m0
(B[0;1mfilters(B[0m (B[0;2m·········(B[0m = [34m[]
(B[0;1mforce(B[0m (B[0;2m···········(B[0m = [31mfalse
(B[0;1mforce_recursion(B[0m[0;2m · (B[0m= [31mfalse
(B[0;1mheaders(B[0m (B[0;2m·········(B[0m = [34m[]
(B[0;1mhttp1(B[0m (B[0;2m···········(B[0m = [31mfalse
(B[0;1mhttp2(B[0m (B[0;2m···········(B[0m = [31mfalse
(B[0;1mlist(B[0m (B[0;2m············(B[0m = (B[0;2mnull
(B[0;1mmethod(B[0m (B[0;2m··········(B[0m = [92m"GET"
(B[0;1mmode(B[0m (B[0;2m············(B[0m = [92m"recursive"
(B[0;1mno_save(B[0m (B[0;2m·········(B[0m = [31mfalse
(B[0;1moutput(B[0m (B[0;2m··········(B[0m = (B[0;2mnull
(B[0;1mresume(B[0m (B[0;2m··········(B[0m = [31mfalse
(B[0;1mretries(B[0m (B[0;2m·········(B[0m = [93m3
(B[0;1mretry_codes(B[0m (B[0;2m·····(B[0m = [34m[]
(B[0;1mshow(B[0m (B[0;2m············(B[0m = [34m[]
(B[0;1mthreads(B[0m (B[0;2m·········(B[0m = [93m55
(B[0;1mthrottle(B[0m (B[0;2m········(B[0m = (B[0;2mnull
(B[0;1mtransforms(B[0m (B[0;2m······(B[0m = [34m[]
(B[0;1murl(B[0m (B[0;2m·············(B[0m = (B[0;2mnull
(B[0;1mwordlist_filter(B[0m[0;2m · (B[0m= (B[0;2mnull
(B[0;1mwordlists(B[0m (B[0;2m·······(B[0m = [34m[]
```
