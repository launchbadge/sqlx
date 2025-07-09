# Usage of `macros.preferred-crates` in `sqlx.toml`

## The Problem

SQLx has many optional features that enable integrations for external crates to map from/to SQL types.

In some cases, more than one optional feature applies to the same set of types:

* The `chrono` and `time` features enable mapping SQL date/time types to those in these crates.
* Similarly, `bigdecimal` and `rust_decimal` enable mapping for the SQL `NUMERIC` type.

Throughout its existence, the `query!()` family of macros has inferred which crate to use based on which optional 
feature was enabled. If multiple features are enabled, one takes precedent over the other: `time` over `chrono`, 
`rust_decimal` over `bigdecimal`, etc. The ordering is purely the result of historical happenstance and 
does not indicate any specific preference for one crate over another. They each have their tradeoffs.

This works fine when only one crate in the dependency graph depends on SQLx, but can break down if another crate
in the dependency graph also depends on SQLx. Because of Cargo's [feature unification], any features enabled
by this other crate are also forced on for all other crates that depend on the same version of SQLx in the same project.

This is intentional design on Cargo's part; features are meant to be purely additive, so it can build each transitive
dependency just once no matter how many crates depend on it. Otherwise, this could result in combinatorial explosion.

Unfortunately for us, this means that if your project depends on SQLx and enables the `chrono` feature, but also depends 
on another crate that enables the `time` feature, the `query!()` macros will end up thinking that _you_ want to use 
the `time` crate, because they don't know any better. 

Fixing this has historically required patching the dependency, which is annoying to maintain long-term.

[feature unification]: https://doc.rust-lang.org/cargo/reference/features.html#feature-unification

## The Solution

However, as of 0.9.0, SQLx has gained the ability to configure the macros through the use of a `sqlx.toml` file.

This includes the ability to tell the macros which crate you prefer, overriding the inference.

See the [`sqlx.toml`](./sqlx.toml) file in this directory for details.

A full reference `sqlx.toml` is also available as `sqlx-core/src/config/reference.toml`.

## This Example

This example exists both to showcase the macro configuration and also serve as a test for the functionality.

It consists of three crates:

* The root crate, which depends on SQLx and enables the `chrono` and `bigdecimal` features,
* `uses-rust-decimal`, a dependency which also depends on SQLx and enables the `rust_decimal` feature,
* and `uses-time`, a dependency which also depends on SQLx and enables the `time` feature.
  * This serves as a stand-in for `tower-sessions-sqlx-store`, which is [one of the culprits for this issue](https://github.com/launchbadge/sqlx/issues/3412#issuecomment-2277377597).

Given that both dependencies enable features with higher precedence, they would historically have interfered
with the usage in the root crate. (Pretend that they're published to crates.io and cannot be easily changed.) 
However, because the root crate uses a `sqlx.toml`, the macros know exactly which crates it wants to use and everyone's happy.
