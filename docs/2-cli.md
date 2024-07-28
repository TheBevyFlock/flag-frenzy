# CLI Usage

A full list of these arguments can be found using:

```bash
flag-frenzy --help
```

## Checking a project

By default, `flag-frenzy` will check all crates in the current workspace, loading its configuration from `./config`. You can specify a path to a custom `Cargo.toml` and config folder using `--manifest-path` and `--config`:

```bash
flag-frenzy --manifest-path path/to/Cargo.toml --config path/to/config
```

It will then go through each crate and test all of its feature combinations, conforming to the rules in the configuration folder.

## Checking a single crate

`flag-frenzy` by default checks all crates in a workspace. If you want to check just one crate, use the `--package` option:

```bash
flag-frenzy --manifest-path bevy/Cargo.toml --package bevy_ecs
# Or
flag-frenzy --manifest-path bevy/Cargo.toml --p bevy_ecs
```

## Dividing up work

If you want to split feature checking across multiple devices, you can use work chunking. Chunking will divide all of the crates in a workspace into "chunks", which can then be checked separate from each other.

```bash
flag-frenzy --total-chunks 5 --chunk 0
```

`--chunk` specifies which chunk will be checked. Note that it starts counting from 0, so in the above example 0 is the first chunk and 4 is the last.

Chunks are split up in a best-effort way, so that they all have similar runtimes. For example, let's say you have two chunks and want to check three crates:

|Crate|Feature Combinations|
|-|-|
|`foo`|100|
|`bar`|50|
|`baz`|50|

In this case, chunk 0 will just check `foo` while chunk 0 will check both `bar` and `baz`. Please note that the combinations calculation is an estimate that does not account of configuration or rules.

## Enabling / disabling colorful output

`flag-frenzy` by default uses ANSI escape codes to make its terminal output colorful. If you are running it on a terminal that does not support these colors, or piping the output to a file, you can disable it using the `--color` option:

```bash
flag-frenzy --color never
```
