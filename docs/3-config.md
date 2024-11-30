# Configuration

Configuration is stored in the `config` folder, though can be loaded from another folder using the `--config` option. Configuration is specified using a series of [TOML](https://toml.io) files. For example, the `config` folder may look like this:

```
config/
- global.toml
- bevy.toml
- bevy_ecs.toml
- bevy_render.toml
```

## Naming

A TOML file named `bevy_ecs.toml` will only affect the `bevy_ecs` crate, and a file named `bevy.toml` will only affect the `bevy` crate. The name of a file specifies which crate it is associated with. The only exception to this rule is `global.toml`.

`global.toml` can override the default configuration for **all** crates. If `bevv_ecs.toml` does not specify `max_combo_size`, for example, `flag-frenzy` will fall back to `max_combo_size` in `global.toml` before using the hard-coded constant. Note that `global.toml` cannot specify rules; `flag-frenzy` will throw an error.

## Options

### `max_combo_size`

`max_combo_size` is an optional integer that specifies the maximum amount of features allowed in a single combination. This is a way to decrease the runtime for crates with dozens of features. For instance, take a look at this `Cargo.toml`:

```toml
[features]
red = []
green = []
blue = []
alpha = []
```

If `flag-frenzy` were to check all combinations, it would have to run `cargo check` $nCr(4, 0) + nCr(4, 1) + nCr(4, 2) + nCr(4, 3) + nCr(4, 4) = 16$ times. By setting `max_combo_size = 2`, it would decrease the amount of combinations to $nCr(4, 0) + nCr(4, 1) + nCr(4, 2) = 11$ times. While not necessary for small crates, this is lifesaving when you have dozens or hundreds of features.

While decrease the maximum combo size can save time, it technically will not test all combinations. In practice, this is fine. You usually only need 2-3 features to find most bugs.

## `skip_optional_deps`

Crates can specify optional dependencies that are treated as features by Cargo:

```toml
[dependencies]
serde = { version = "1", optional = true }
```

For some crates, it may not be necessary to check combinations of optional dependencies. You can skip them entirely by setting `skip_optional_deps = true`.

## `skip_used_optional_deps`

Crates can specify optional dependencies and use them in features, or leave them as an implicit or explicit feature. 

```toml
[dependencies]
used = { path = "../used", optional = true }
implicit = { path = "../unused", optional = true }
explicit = { path = "../explicit", optional = true }

[features]
foo = ["used"]
explicit = ["dep:explicit"]
```

For some crates, it may not be necessary to check the usage of used features. You can skip them entirely by setting `skip_used_optional_deps = true`.