+++
title = "Wordlists"
weight = 20
+++

What would a web fuzzer be without wordlists? 

## Named wordlists

Assigning names allows you to reference your wordlists later in other options, such as [filters](@/docs/filters.md#wordlist-filters) (`--wf`) or in the URL directly.

When specifying a wordlist, you can use the following syntax:

```ansi
[2m[1mrwalk[0m [2mhttps://example.com[0m [32mpath/to/wordlist[0m:[33m[4mALIAS
```

And then use it:

```ansi
[2m[1mrwalk[0m [2mhttps://example.com/[0m[33m[4mALIAS[0m [2m[32mpath/to/wordlist[0m[2m:[0m[33m[4mALIAS[0m[2m --mode template[0m
```

or

```ansi
[2m[1mrwalk[0m [2mhttps://example.com/[0m [2m[32mpath/to/wordlist[0m:[0m[34m[4mALIAS[0m --wf [0m[[34m[4mALIAS[0m][33mlength[0m:[32m<5[0m
```

## On-the-fly modifications

Transformations can be applied to each wordlist entry at runtime with the `--transform` (or `-t`) option. Specify more of them by separating with a `;`.

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
            <td><code>case</code></td>
            <td><code>c</code></td>
            <td>

```ansi
[33mcase[0m:[32mlower[0m, [33mcase[0m:[32mlower[0m, [33mcase[0m:[32mtitle[0m
```   

</td>
        </tr>
        <tr>
            <td><code>replace</code></td>
            <td><code>rp</code>, <code>sub</code></td>
            <td>

```ansi
[33mreplace[0m:[32mfoo[0m, [33mreplace[0m:[32mbar[0m
```

</td>
        </tr>
        <tr>
            <td><code>remove</code></td>
            <td><code>r</code>, <code>rm</code></td>
            <td>

```ansi
[33mremove[0m:[32mfoo[0m
```
</td>
        </tr>
        <tr>
            <td><code>prefix</code></td>
            <td><code>p</code></td>
            <td>

```ansi
[33mprefix[0m:[32mfoo[0m
```

</td>
        </tr>
        <tr>
            <td><code>suffix</code></td>
            <td><code>s</code></td>
            <td>

```ansi
[33msuffix[0m:[32mbar[0m
```
</td>
        </tr>
        <tr>
            <td><code>encode</code></td>
            <td><code>e</code>, <code>enc</code></td>
            <td>

```ansi
[33mencode[0m:[32murl[0m, [33mencode[0m:[32mbase64[0m, [33mencode[0m:[32mhex[0m
```

</td>
        </tr>
    </tbody>
</table>

## Merging wordlists

Need to combine wordlists? Use the `--merge` (or `--mw`) option:

```ansi
[2m[1mrwalk[0m [2m...[0m [32mlist1.txt[0m:[33mfoo[0m [32mlist2.txt[0m:[33mbar[0m [2m--merge[0m [33mfoo[0m,[33mbar[0m:[33mcombined[0m
```

This creates a new wordlist with alias `combined` containing all entries from both foo and bar.

> [!info]
> Merging is done after filtering and transformations. 
> This allows you to create variations of a wordlist and use them together. 
> 
> For example, to fuzz with both uppercase and lowercase versions:
> ```ansi
> [2m[1mrwalk[0m [2m...[0m [32mlist.txt[0m:[33mUP[0m [32mlist.txt[0m:[33mLO[0m -t "[0m[[34mUP[0m][33mcase[0m:[32mupper[0m; [0m[[34mLO[0m][33mcase[0m:[32mlower[0m" \ 
>   --merge [33mUP,LO[0m:[33mMERGED[0m
> ```


## Comments in wordlists

`rwalk` automatically:

- Skips empty lines
- Ignores lines starting with `#`
- Trims comments after a **space** and `#` character

If you want to include comment lines, use the `--include-comments` flag.