# `flag-frenzy`

`flag-frenzy` is an advanced feature flag testing tool for [Cargo](https://doc.rust-lang.org/cargo/index.html), perfect for large projects and continuous integration. It checks combinations of feature flags for crates within a workspace, filtering them by configurable rules.

You may be interested in these great, simpler alternatives:

- [`cargo-hack`](https://crates.io/crates/cargo-hack)
- [`cargo-all-features`](https://lib.rs/crates/cargo-all-features)

## History

`flag-frenzy` was created to test feature flags for crates within the [Bevy game engine](https://bevyengine.org). Originally it was a Github Actions job that ran `cargo-all-features` ([source](https://github.com/TheBevyFlock/flag-frenzy/tree/5eb37225b517566159aa4e215bebb01424b36769)), but quickly issues arose with out-of-memory errors and insufficient feature configuration.

## Features

The `flag-frenzy` you know of today was written from scratch with the previous section's issues in mind. Specifically, it offers:

- Lazy combinations with [`Iterator`](https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html)
    - Since combinations are exponential, you would run out of memory if you tried to compute them all ahead of time. Instead, `flag-frenzy` uses an `Iterator` to only compute a combination when it is needed, and drops the value when it is done.
- Expressive feature configuration with rules
    - Easily express complex feature requirements, such as: "`feature1` requires `feature2` or `feature3`" and "`feature4` and `feature5` are incompatible, unless `feature6` is enabled."
    - This allows you to test only unintended behavior with feature flags, and skip the combinations that you intended to not work.
- Split up work into chunks that can be distributed across multiple processes in parallel.
    - This was re-implemented from `cargo-all-features` because it drastically decreases the time required is check a workspace.
    - Note that this is on a best-effort basis: individual crates cannot be subdivided, and it does not account for rules and other filters. See [#14](https://github.com/TheBevyFlock/flag-frenzy/issues/14) for more information.
- Colorful output and failure reports that help diagnose exactly which combinations raise errors.

## Install

`flag-frenzy` is primarily geared to testing Bevy using Github Actions. Because of this, it does not have version numbers or changelogs, and it is not published on <https://crates.io>. You must install it from the repository:

```bash
cargo install --git https://github.com/TheBevyFlock/flag-frenzy.git
```

To ensure that it installed, try running it:

```bash
flag-frenzy --help
```

## Usage

TODO :)

## Trophies :trophy:

- [#14298](https://github.com/bevyengine/bevy/pull/14298) `bevy_window` failing with `serialize` feature.
- [#14430](https://github.com/bevyengine/bevy/pull/14430) `bevy_ui` failing without `bevy_text`.
- [#14469](https://github.com/bevyengine/bevy/pull/14469) `bevy_winit` failing without `serialize` feature.
- [#14486](https://github.com/bevyengine/bevy/pull/14486) `bevy_gltf` failing with `pbr_multi_layer_material_textures` or `pbr_anisotropy_texture`.
