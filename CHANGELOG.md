# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.4 - 2020-01-18

### Fixed

 - Fix decoding of Rows containing NULLs in MySQL (and add an integration test so this doesn't break again)

## 0.2.3 - 2020-01-18

### Fixed

 - Fix `query!` when used on a query that does not return results

## 0.2.2 - 2020-01-16

### Added

 - [[#57]] Add support for unsigned integers and binary types in `query!` for MySQL [[@mehcode]]

[#57]: https://github.com/launchbadge/sqlx/issues/57

### Fixed

 - Fix stall when requesting TLS from a Postgres server that explicitly does not support TLS (such as postgres running inside docker) [[@abonander]]

 - [[#66]] Declare used features for `tokio` in `sqlx-macros` explicitly
 
[#66]: https://github.com/launchbadge/sqlx/issues/66

## 0.2.1 - 2020-01-16

### Fixed

 - [[#64], [#65]] Fix decoding of Rows containing NULLs in MySQL [[@danielakhterov]]

[#64]: https://github.com/launchbadge/sqlx/pull/64
[#65]: https://github.com/launchbadge/sqlx/pull/65

 - [[#55]] Use a shared tokio runtime for the `query!` macro compile-time execution (under the `runtime-tokio` feature) [[@udoprog]] 

[#55]: https://github.com/launchbadge/sqlx/pull/55

## 0.2.0 - 2020-01-15

### Fixed

 - https://github.com/launchbadge/sqlx/issues/47
 
### Added

 - Support Tokio through an optional `runtime-tokio` feature.

 - Support SQL transactions. You may now use the `begin()` function on `Pool` or `Connection` to 
   start a new SQL transaction. This returns `sqlx::Transaction` which will `ROLLBACK` on `Drop`
   or can be explicitly `COMMIT` using `commit()`.
   
 - Support TLS connections.

## 0.1.4 - 2020-01-11

### Fixed

 - https://github.com/launchbadge/sqlx/issues/43

 - https://github.com/launchbadge/sqlx/issues/40

### Added

 - Support for `SCRAM-SHA-256` authentication in Postgres [#37](https://github.com/launchbadge/sqlx/pull/37) [@danielakhterov](https://github.com/danielakhterov)

 - Implement `Debug` for Pool [#42](https://github.com/launchbadge/sqlx/pull/42) [@prettynatty](https://github.com/prettynatty)

## 0.1.3 - 2020-01-06

### Fixed

 - https://github.com/launchbadge/sqlx/issues/30

## 0.1.2 - 2020-01-03

### Added
 
 - Support for Authentication in MySQL 5+ including the newer authentication schemes now default in MySQL 8: `mysql_native_password`, `sha256_password`, and `caching_sha2_password`.

 - [`Chrono`](https://github.com/chronotope/chrono) support for MySQL was only partially implemented (was missing `NaiveTime` and `DateTime<Utc>`).

 - `Vec<u8>` (and `[u8]`) support for MySQL (`BLOB`) and Postgres (`BYTEA`).

[@abonander]: https://github.com/abonander
[@danielakhterov]: https://github.com/danielakhterov
[@mehcode]: https://github.com/mehcode
[@udoprog]: https://github.com/udoprog
