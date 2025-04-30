+++
title = "Filters"
weight = 10
+++

Filters are a must-have when it comes to bruteforcing stuff. They come in two flavors with `rwalk`: **response** (`--filter`) and **wordlist** (`--wf`) filters. 

Composing filters may seem a bit overwhelming, but once you get the hang of it, you'll find that they are quite powerful. 

The basic syntax for a single filter is:

```
name:argument
```

You can obviously use multiple filters together, by using boolean operators:

| Operator | Name  |
| :------: | ----- |
|   `&`    | `AND` |
|   `\|`   | `OR`  |
|   `!`    | `NOT` |

```
(filter1:arg1 & filter2:arg2) | filter3:arg3
```

## Response filters

<table>
    <thead>
        <tr>
            <th>Name</th>
            <th>Aliases</th>
            <th>Usage</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>contains</code></td>
            <td><code>c</code></td>
            <td>

```ansi
[032mcontains[0m:[033msubstring
```
            
</td>
        </tr>
        <tr>
            <td><code>ends</code></td>
            <td><code>e</code></td>
            <td>

```ansi
[032mends[0m:[033msubstring
```
            
</td>
        </tr>
        <tr>
            <td><code>starts</code></td>
            <td><code>st</code></td>
            <td>

```ansi
[032mstarts[0m:[033msubstring
```
            
</td>
        </tr>
        <tr>
            <td><code>length</code></td>
            <td><code>l</code>, <code>size</code></td>
            <td>

```ansi
[032mlength[0m:[033m>123[0m, [032mlength[0m:[033m123-456
```
            
</td>
        </tr>
        <tr>
            <td><code>regex</code></td>
            <td><code>r</code></td>
            <td>

```ansi
[032mregex[0m:[033m(a|b).*
```
            
</td>
        </tr>
        <tr>
            <td><code>status</code></td>
            <td><code>s</code>, <code>code</code></td>
            <td>

```ansi
[032mstatus[0m:[033m200[0m, [032mstatus[0m[033m<400
```
            
</td>
        </tr>
        <tr>
            <td><code>time</code></td>
            <td><code>d</code>, <code>duration</code></td>
            <td>

```ansi
[032mtime[0m:[033m>100ms[0m, [032mtime[0m:[033m1s-2s
```
            
</td>
        </tr>
        <tr>
            <td><code>script</code></td>
            <td><code>rhai</code></td>
            <td>

```ansi
[032mscript[0m:[033m/path/to/script.rhai
```
            
</td>
        </tr>
        <tr>
            <td><code>type</code></td>
            <td><code>t</code></td>
            <td>

```ansi
[032mtype[0m:[033mdir[0m, [032mtype[0m:[033mtext/html[0m, [032mtype[0m:[033merr
```
            
</td>
        </tr>
        <tr>
            <td><code>header</code></td>
            <td><code>h</code></td>
            <td>

```ansi
[032mheader[0m:[033maccept=application/json
```
            
</td>
        </tr>
    </tbody>
</table>

## Wordlist filters


<table>
    <thead>
        <tr>
            <th>Name</th>
            <th>Aliases</th>
            <th>Usage</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>contains</code></td>
            <td><code>c</code></td>
            <td>

```ansi
[032mcontains[0m:[033msubstring
```
            
</td>
        </tr>
        <tr>
            <td><code>ends</code></td>
            <td><code>e</code></td>
            <td>

```ansi
[032mends[0m:[033msubstring
```
            
</td>
        </tr>
        <tr>
            <td><code>starts</code></td>
            <td><code>st</code></td>
            <td>

```ansi
[032mstarts[0m:[033msubstring
```
            
</td>
        </tr>
        <tr>
            <td><code>length</code></td>
            <td><code>l</code>, <code>size</code></td>
            <td>

```ansi
[032mlength[0m:[033m>123[0m, [032mlength[0m:[033m123-456
```
            
</td>
        </tr>
        <tr>
            <td><code>regex</code></td>
            <td><code>r</code></td>
            <td>

```ansi
[032mregex[0m:[033m(a|b).*
```
            
</td>
        </tr>
    </tbody>
</table>