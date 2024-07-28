# Rules

Rules are constraints that can be defined in configuration files to represent complex feature relationships.

## Feature expression

Rules are built on the concept of feature expressions, which evaulate to true or false depending on if specified features are within a combination. Here are a series of example expressions, in TOML, with comments explaining what they do:

```toml
# True if `foo` is enabled.
"foo"
# Same as previous expression.
["foo"]
# True if `foo` AND `bar` are enabled.
["foo", "bar"]
# True if `foo` OR `bar` is enabled.
["foo", "OR", "bar"]
# True if `foo` is enabled, OR if `bar` and `baz` are enabled.
["foo", "OR", ["bar", "baz"]]
```

Feature expressions can be nested infinitely, and can represent complex relations through the AND and OR operators.

## Rule reference

Rules are specified using arrays of tables. They specify whether a combination of features should be checked or skipped.

```toml
[[rule]]
# If true, then this rule will evaluated. This can either be a feature expression or the `true`
# literal.
when = "expression"
# Optionally specifies features that must be present. If it evaluates to false, this combination
# will be skipped.
require = "expression"
# Optionally specifies features that must NOT be present. If it evaluates to true, this combination
# will be skipped. This can either be a feature expression or the `true` literal.
forbid = "expression"
```

## Patterns

### Skipping features

```toml
# Always skips `foo`.
[[rule]]
when = true
forbid = "foo"

# Alternative
[[rule]]
when = "foo"
forbid = true
```

### Requiring features

```toml
# `foo` and `bar` are always required.
[[rule]]
when = true
require = ["foo", "bar"]
```

### Choosing one of multiple features

```toml
# When `rendering` is enabled, require the `2d` or `3d` backend.
[[rule]]
when = "rendering"
require = ["2d", "OR", "3d"]
```

Note that, while the above rule will pass for `["rendering", "2d"]` and `["rendering", "3d"]`, it will _also_ pass for `["rendering", "2d", "3d"]`. If you need `2d` and `3d` to be incompatible with each other, you can use:

```toml
[[rule]]
when = "rendering"
require = ["2d", "OR", "3d"]
# Forbid both from being enabled at the same time.
forbid = ["2d", "3d"]
```
