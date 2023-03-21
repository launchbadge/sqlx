# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### 0.6.3 - 2023-03-21

This is a hotfix to address the breakage caused by transitive dependencies upgrading to `syn = "2"`.

We set `default-features = false` for our dependency on `syn = "1"` to be good crates.io citizens, 
but failed to enable the features we actually used, which went undetected because we transitively depended on
`syn` with the default features enabled through other crates, 
and so they were also on for us because features are additive.

When those other dependencies upgraded to `syn = "2"` it was no longer enabling those features for us, 
and so compilation broke for projects that don't also depend on `syn = "1"`, transitively or otherwise.

There is no PR for this fix as there was no longer a dedicated development branch for `0.6`, 
but discussion can be found in [issue #2418].

As of this release, the `0.7` release is in alpha and so development is no longer occurring against `0.6`.
This fix will be forward-ported to `0.7`.

[issue #2418]: https://github.com/launchbadge/sqlx/issues/2418

## 0.6.2 - 2022-09-14

[25 pull requests][0.6.2-prs] were merged this release cycle.

### Added
* [[#1081]]: Add `try_from` attribute for `FromRow` derive [[@zzhengzhuo]]
    * Exemplifies "out of sight, out of mind." It's surprisingly easy to forget about PRs when they get pushed onto
      the second page. We'll be sure to clean out the backlog for 0.7.0.
* [[#2014]]: Support additional SQLCipher options in SQLite driver. [[@szymek156]]
* [[#2052]]: Add issue templates [[@abonander]]
* [[#2053]]: Add documentation for `IpAddr` support in Postgres [[@rakshith-ravi]]
* [[#2062]]: Add extension support for SQLite [[@bradfier]]
* [[#2063]]: customizable db locking during migration [[@fuzzbuck]]

### Changed
* [[#2025]]: Bump sqlformat to 2.0 [[@NSMustache]]
* [[#2056]]: chore: Switch to sha1 crate [[@stoically]]
* [[#2071]]: Use cargo check consistently in `prepare` [[@cycraig]]

### Fixed
* [[#1991]]: Ensure migration progress is not lost for Postgres, MySQL and SQLite. [[@crepererum]]
* [[#2023]]: Fix expansion of `#[sqlx(flatten)]` for `FromRow` derive [[@RustyYato]]
* [[#2028]]: Use fully qualified path when forwarding to `#[test]` from `#[sqlx::test]` [[@alexander-jackson]]
* [[#2040]]: Fix typo in `FromRow` docs [[@zlidner]]
* [[#2046]]: added flag for PIPES_AS_CONCAT connection setting for MySQL to fix #2034 [[@marcustut]]
* [[#2055]]: Use unlock notify also on `sqlite3_exec`  [[@madadam]]
* [[#2057]]: Make begin,commit,rollback cancel-safe in sqlite  [[@madadam]]
* [[#2058]]: fix typo in documentation [[@lovasoa]]
* [[#2067]]: fix(docs): close code block in query_builder.rs [[@abonander]]
* [[#2069]]: Fix `prepare` race condition in workspaces [[@cycraig]]
* [[#2072]]: SqliteConnectOptions typo [[@fasterthanlime]]
* [[#2074]]: fix: mssql uses unsigned for tinyint instead of signed [[@he4d]]
* [[#2081]]: close unnamed portal after each executed extended query [[@DXist]]
* [[#2086]]: PgHasArrayType for transparent types fix. [[@Wopple]]
    * NOTE: this is a breaking change and has been postponed to 0.7.0.
* [[#2089]]: fix: Remove default chrono dep on time for sqlx-cli [[@TravisWhitehead]]
* [[#2091]]: Sqlite explain plan log efficiency [[@tyrelr]]

[0.6.2-prs]: https://github.com/launchbadge/sqlx/pulls?q=is%3Apr+is%3Aclosed+merged%3A2022-08-04..2022-09-14+

[#1081]: https://github.com/launchbadge/sqlx/pull/1081
[#1991]: https://github.com/launchbadge/sqlx/pull/1991
[#2014]: https://github.com/launchbadge/sqlx/pull/2014
[#2023]: https://github.com/launchbadge/sqlx/pull/2023
[#2025]: https://github.com/launchbadge/sqlx/pull/2025
[#2028]: https://github.com/launchbadge/sqlx/pull/2028
[#2040]: https://github.com/launchbadge/sqlx/pull/2040
[#2046]: https://github.com/launchbadge/sqlx/pull/2046
[#2052]: https://github.com/launchbadge/sqlx/pull/2052
[#2053]: https://github.com/launchbadge/sqlx/pull/2053
[#2055]: https://github.com/launchbadge/sqlx/pull/2055
[#2056]: https://github.com/launchbadge/sqlx/pull/2056
[#2057]: https://github.com/launchbadge/sqlx/pull/2057
[#2058]: https://github.com/launchbadge/sqlx/pull/2058
[#2062]: https://github.com/launchbadge/sqlx/pull/2062
[#2063]: https://github.com/launchbadge/sqlx/pull/2063
[#2067]: https://github.com/launchbadge/sqlx/pull/2067
[#2069]: https://github.com/launchbadge/sqlx/pull/2069
[#2071]: https://github.com/launchbadge/sqlx/pull/2071
[#2072]: https://github.com/launchbadge/sqlx/pull/2072
[#2074]: https://github.com/launchbadge/sqlx/pull/2074
[#2081]: https://github.com/launchbadge/sqlx/pull/2081
[#2086]: https://github.com/launchbadge/sqlx/pull/2086
[#2089]: https://github.com/launchbadge/sqlx/pull/2089
[#2091]: https://github.com/launchbadge/sqlx/pull/2091

## 0.6.1 - 2022-08-02

[33 pull requests][0.6.1-prs] were merged this release cycle.

### Added
* [[#1495]]: Add example for manual implementation of the `FromRow` trait [[@Erik1000]]
* [[#1822]]: (Postgres) Add support for `std::net::IpAddr` [[@meh]]
    * Decoding returns an error if the `INET` value in Postgres is a prefix and not a full address
      (`/32` for IPv4, `/128` for IPv6).
* [[#1865]]: Add SQLite support for the `time` crate [[@johnbcodes]]
* [[#1902]]: Add an example of how to use `QueryBuilder::separated()` [[@sbeckeriv]]
* [[#1917]]: Added docs for `sqlx::types::Json` [[@jayy-lmao]]
* [[#1919]]: Implement `Clone` for `PoolOptions` [[@Thomasdezeeuw]]
* [[#1953]]: Support Rust arrays in Postgres [[@e00E]]
* [[#1954]]: Add `push_tuples` for `QueryBuilder` [[@0xdeafbeef]]
* [[#1959]]: Support `#[sqlx(flatten)]` attribute in `FromRow` [[@TheoOiry]]
* [[#1967]]: Add example with external query files [[@JoeyMckenzie]]
* [[#1985]]: Add `query_builder::Separated::push_bind_unseparated()` [[@0xdeafbeef]]
* [[#2001]]: Implement `#[sqlx::test]` for general use
    * Includes automatic database management, migration and fixture application.
    * Drops support for end-of-lifed database versions, see PR for details.
* [[#2005]]: `QueryBuilder` improvements [[@abonander]]
    * Raw SQL getters, new method to build `QueryAs` instead of `Query`.
* [[#2013]]: (SQLite) Allow VFS to be set as URL query parameter [[@liningpan]] 

### Changed
* [[#1679]]: refactor: alias actix-* features to their equivalent tokio-* features [[@robjtede]]
* [[#1906]]: replaced all uses of "uri" to "url" [[@RomainStorai]]
* [[#1965]]: SQLite improvements [[@abonander]]
* [[#1977]]: Docs: clarify relationship between `query_as!()` and `FromRow` [[@abonander]]
* [[#2003]]: Replace `dotenv` with `dotenvy` [[@abonander]]

### Fixed
* [[#1802]]: Try avoiding a full clean in `cargo sqlx prepare --merged` [[@LovecraftianHorror]]
* [[#1848]]: Fix type info access in `Any` database driver [[@raviqqe]]
* [[#1910]]: Set `CARGO_TARGET_DIR` when compiling queries [[@sedrik]]
* [[#1915]]: Pool: fix panic when using callbacks [[@abonander]]
* [[#1930]]: Don't cache SQLite connection for macros [[@LovecraftianHorror]]
* [[#1948]]: Fix panic in Postgres `BYTEA` decode [[@e00E]]
* [[#1955]]: Fix typo in FAQ [[@kenkoooo]]
* [[#1968]]: (Postgres) don't panic if `S` or `V` notice fields are not UTF-8 [[@abonander]]
* [[#1969]]: Fix sqlx-cli build [[@ivan]]
* [[#1974]]: Use the `rust-cache` action for CI [[@abonander]]
* [[#1988]]: Agree on a single default runtime for the whole workspace [[@crepererum]]
* [[#1989]]: Fix panics in `PgListener` [[@crepererum]]
* [[#1990]]: Switch `master` to `main` in docs [[@crepererum]]
    * The change had already been made in the repo, the docs were out of date.
* [[#1993]]: Update versions in quickstart examples in README [[@UramnOIL]]

[0.6.1-prs]: https://github.com/launchbadge/sqlx/pulls?page=1&q=is%3Apr+is%3Aclosed+merged%3A2022-06-17..2022-08-02

[#1906]: https://github.com/launchbadge/sqlx/pull/1906
[#1495]: https://github.com/launchbadge/sqlx/pull/1495
[#1679]: https://github.com/launchbadge/sqlx/pull/1679
[#1802]: https://github.com/launchbadge/sqlx/pull/1802
[#1822]: https://github.com/launchbadge/sqlx/pull/1822
[#1848]: https://github.com/launchbadge/sqlx/pull/1848
[#1865]: https://github.com/launchbadge/sqlx/pull/1865
[#1902]: https://github.com/launchbadge/sqlx/pull/1902
[#1910]: https://github.com/launchbadge/sqlx/pull/1910
[#1915]: https://github.com/launchbadge/sqlx/pull/1915
[#1917]: https://github.com/launchbadge/sqlx/pull/1917
[#1919]: https://github.com/launchbadge/sqlx/pull/1919
[#1930]: https://github.com/launchbadge/sqlx/pull/1930
[#1948]: https://github.com/launchbadge/sqlx/pull/1948
[#1953]: https://github.com/launchbadge/sqlx/pull/1953
[#1954]: https://github.com/launchbadge/sqlx/pull/1954
[#1955]: https://github.com/launchbadge/sqlx/pull/1955
[#1959]: https://github.com/launchbadge/sqlx/pull/1959
[#1965]: https://github.com/launchbadge/sqlx/pull/1965
[#1967]: https://github.com/launchbadge/sqlx/pull/1967
[#1968]: https://github.com/launchbadge/sqlx/pull/1968
[#1969]: https://github.com/launchbadge/sqlx/pull/1969
[#1974]: https://github.com/launchbadge/sqlx/pull/1974
[#1977]: https://github.com/launchbadge/sqlx/pull/1977
[#1985]: https://github.com/launchbadge/sqlx/pull/1985
[#1988]: https://github.com/launchbadge/sqlx/pull/1988
[#1989]: https://github.com/launchbadge/sqlx/pull/1989
[#1990]: https://github.com/launchbadge/sqlx/pull/1990
[#1993]: https://github.com/launchbadge/sqlx/pull/1993
[#2001]: https://github.com/launchbadge/sqlx/pull/2001
[#2003]: https://github.com/launchbadge/sqlx/pull/2003
[#2005]: https://github.com/launchbadge/sqlx/pull/2005
[#2013]: https://github.com/launchbadge/sqlx/pull/2013

## 0.6.0 - 2022-06-16

This release marks the end of the 0.5.x series of releases and contains a number of breaking changes,
mainly to do with backwards-incompatible dependency upgrades. 

As we foresee many more of these in the future, we [surveyed the community] on how to handle this;
the consensus appears to be "just release breaking changes more often." 

As such, we expect the 0.6.x release series to be a shorter one.

[39 pull requests(!)][0.6.0-prs] (not counting "prepare 0.5.12 release", of course) were merged this release cycle.

### Breaking
* [[#1384]]: (Postgres) Move `server_version_num` from trait to inherent impl [[@AtkinsChang]]
* [[#1426]]: Bump `ipnetwork` to 0.19 [[@paolobarbolini]]
* [[#1455]]: Upgrade `time` to 0.3 [[@paolobarbolini]]
* [[#1505]]: Upgrade `rustls` to 0.20 [[@paolobarbolini]]
    * Fortunately, future upgrades should not be breaking as `webpki` is no longer exposed in the API.
* [[#1529]]: Upgrade `bigdecimal` to 0.3 [[@e00E]]
* [[#1602]]: postgres: use `Oid` everywhere instead of `u32` [[@paolobarbolini]]
    * This drops the `Type`, `Decode`, `Encode` impls for `u32` for Postgres as it was misleading.
      Postgres doesn't support unsigned ints without using an extension. These impls were decoding Postgres `OID`s
      as bare `u32`s without any context (and trying to bind a `u32` to a query would produce an `OID` value in SQL).
      This changes that to use a newtype instead, for clarity.
* [[#1612]]: Make all `ConnectOptions` types cloneable [[@05storm26]]
* [[#1618]]: SQLite `chrono::DateTime<FixedOffset>` timezone fix [[@05storm26]]
    * `DateTime<FixedOffset>` will be stored in SQLite with the correct timezone instead of always in UTC.
      This was flagged as a "potentially breaking change" since it changes how dates are sent to SQLite.
* [[#1733]]: Update `git2` to 0.14 [[@joshtriplett]]
* [[#1734]]: Make `PgLTree::push()` infallible and take `PgLTreeLabel` directly [[@sebpuetz]]
* [[#1785]]: Fix Rust type for SQLite `REAL` [[@pruthvikar]]
    * Makes the macros always map a `REAL` column to `f64` instead of `f32` as SQLite uses **only** 64-bit floats.
* [[#1816]]: Improve SQLite support for sub-queries and CTEs [[@tyrelr]]
    * This likely will change the generated code for some invocations `sqlx::query!()` with SQLite.
* [[#1821]]: Update `uuid` crate to v1 [[@paolobarbolini]]
* [[#1901]]: Pool fixes and breaking changes [[@abonander]]
    * Renamed `PoolOptions::connect_timeout` to `acquire_timeout` for clarity.
    * Changed the expected signatures for `PoolOptions::after_connect`, `before_acquire`, `after_release`
    * Changed the signature for `Pool::close()` slightly
        * Now eagerly starts the pool closing, `.await`ing is only necessary if you want to ensure a graceful shutdown.
    * Deleted `PoolConnection::release()` which was previously deprecated in favor of `PoolConnection::detach()`.
    * Fixed connections getting leaked even when calling `.close()`.
* [[#1748]]: Derive `PgHasArrayType` for `#[sqlx(transparent)]` types [[@carols10cents]]
    * This change was released with 0.5.12 but [we didn't realize it was a breaking change] at the time.  
      It was reverted in 0.5.13 and postponed until this release.

### Added
* [[#1843]]: Expose some useful methods on `PgValueRef` [[@mfreeborn]]
* [[#1889]]: SQLx-CLI: add `--connect-timeout` [[@abonander]]
    * Adds a default 10 second connection timeout to all commands.
* [[#1890]]: Added test for mssql LoginAck [[@walf443]]
* [[#1891]]: Added test for mssql ProtocolInfo [[@walf443]]
* [[#1892]]: Added test for mssql ReturnValue [[@walf443]]
* [[#1895]]: Add support for `i16` to `Any` driver [[@EthanYuan]]
* [[#1897]]: Expose `ConnectOptions` and `PoolOptions` on `Pool` and database name on `PgConnectOptions` [[@Nukesor]]

### Changed
* [[#1782]]: Reuse a cached DB connection instead of always opening a new one for `sqlx-macros` [[@LovecraftianHorror]]
* [[#1807]]: Bump remaining dependencies [[@paolobarbolini]]
* [[#1808]]: Update to edition 2021 [[@paolobarbolini]]
    * Note that while SQLx [does not officially track an MSRV] and only officially supports the latest stable Rust, 
      this effectively places a lower bound of 1.56.0 on the range of versions it may work with.
* [[#1823]]: (sqlx-macros) Ignore deps when getting metadata for workspace root [[@LovecraftianHorror]]
* [[#1831]]: Update `crc` to 3.0 [[@djc]]
* [[#1887]]: query_as: don't stop stream after decoding error [[@lovasoa]]

### Fixed
* [[#1814]]: SQLx-cli README: move `Usage` to the same level as `Install` [[@tobymurray]]
* [[#1815]]: SQLx-cli README: reword "building in offline mode" [[@tobymurray]]
* [[#1818]]: Trim `[]` from host string before passing to TcpStream [[@smonv]]
    * This fixes handling of database URLs with IPv6 hosts.
* [[#1842]]: Fix usage of `serde_json` in macros [[@mfreeborn]]
* [[#1855]]: Postgres: fix panics on unknown type OID when decoding [[@demurgos]] 
* [[#1856]]: MySQL: support COLLATE_UTF8MB4_0900_AI_CI [[@scottwey]]
    * Fixes the MySQL driver thinking text columns are bytestring columns when querying against a Planetscale DB.
* [[#1861]]: MySQL: avoid panic when streaming packets are empty [[@e-rhodes]]
* [[#1863]]: Fix nullability check for inner joins in Postgres [[@OskarPersson]]
* [[#1881]]: Fix `field is never read` warnings on Postgres test [[@walf443]]
* [[#1882]]: Fix `unused result must be used` warnings [[@walf443]]
* [[#1888]]: Fix migration checksum comparison during `sqlx migrate info` [[@mdtusz]]
* [[#1894]]: Fix typos [[@kianmeng]]

[surveyed the community]: https://github.com/launchbadge/sqlx/issues/1796
[0.6.0-prs]: https://github.com/launchbadge/sqlx/pulls?page=2&q=is%3Apr+is%3Amerged+merged%3A2022-04-14..2022-06-16
[does not officially track an MSRV]: /FAQ.md#what-versions-of-rust-does-sqlx-support-what-is-sqlxs-msrv
[we didn't realize it was a breaking change]: https://github.com/launchbadge/sqlx/pull/1800#issuecomment-1099898932

[#1384]: https://github.com/launchbadge/sqlx/pull/1384
[#1426]: https://github.com/launchbadge/sqlx/pull/1426
[#1455]: https://github.com/launchbadge/sqlx/pull/1455
[#1505]: https://github.com/launchbadge/sqlx/pull/1505
[#1529]: https://github.com/launchbadge/sqlx/pull/1529
[#1602]: https://github.com/launchbadge/sqlx/pull/1602
[#1612]: https://github.com/launchbadge/sqlx/pull/1612
[#1618]: https://github.com/launchbadge/sqlx/pull/1618
[#1733]: https://github.com/launchbadge/sqlx/pull/1733
[#1734]: https://github.com/launchbadge/sqlx/pull/1734
[#1782]: https://github.com/launchbadge/sqlx/pull/1782
[#1785]: https://github.com/launchbadge/sqlx/pull/1785
[#1807]: https://github.com/launchbadge/sqlx/pull/1807
[#1808]: https://github.com/launchbadge/sqlx/pull/1808
[#1814]: https://github.com/launchbadge/sqlx/pull/1814
[#1815]: https://github.com/launchbadge/sqlx/pull/1815
[#1816]: https://github.com/launchbadge/sqlx/pull/1816
[#1818]: https://github.com/launchbadge/sqlx/pull/1818
[#1821]: https://github.com/launchbadge/sqlx/pull/1821
[#1823]: https://github.com/launchbadge/sqlx/pull/1823
[#1831]: https://github.com/launchbadge/sqlx/pull/1831
[#1842]: https://github.com/launchbadge/sqlx/pull/1842
[#1843]: https://github.com/launchbadge/sqlx/pull/1843
[#1855]: https://github.com/launchbadge/sqlx/pull/1855
[#1856]: https://github.com/launchbadge/sqlx/pull/1856
[#1861]: https://github.com/launchbadge/sqlx/pull/1861
[#1863]: https://github.com/launchbadge/sqlx/pull/1863
[#1881]: https://github.com/launchbadge/sqlx/pull/1881
[#1882]: https://github.com/launchbadge/sqlx/pull/1882
[#1887]: https://github.com/launchbadge/sqlx/pull/1887
[#1888]: https://github.com/launchbadge/sqlx/pull/1888
[#1889]: https://github.com/launchbadge/sqlx/pull/1889
[#1890]: https://github.com/launchbadge/sqlx/pull/1890
[#1891]: https://github.com/launchbadge/sqlx/pull/1891
[#1892]: https://github.com/launchbadge/sqlx/pull/1892
[#1894]: https://github.com/launchbadge/sqlx/pull/1894
[#1895]: https://github.com/launchbadge/sqlx/pull/1895
[#1897]: https://github.com/launchbadge/sqlx/pull/1897
[#1901]: https://github.com/launchbadge/sqlx/pull/1901

## 0.5.13 - 2022-04-15

This is a hotfix that reverts [#1748] as that was an accidental breaking change:  
the generated `PgHasArrayType` impl conflicts with manual impls of the trait.  
This change will have to wait for 0.6.0.

## 0.5.12 - 2022-04-13 (Yanked; use 0.5.13)
[27 pull requests][0.5.12-prs] were merged this release cycle.

### Added
* [[#1641]]: Postgres: Convenient wrapper for advisory locks [[@abonander]]
* [[#1675]]: Add function to undo migrations [[@jdrouet]]
* [[#1722]]: Postgres: implement `PgHasArrayType` for `serde_json::{Value, RawValue}` [[@abreis]]
* [[#1736]]: Derive `Clone` for `MySqlArguments` and `MssqlArguments` [[@0xdeafbeef]]
* [[#1748]]: Derive `PgHasArrayType` for `#[sqlx(transparent)]` types [[@carols10cents]]
* [[#1754]]: Include affected rows alongside returned rows in query logging [[@david-mcgillicuddy-moixa]]
* [[#1757]]: Implement `Type` for `Cow<str>` for MySQL, MSSQL and SQLite [[@ipetkov]]
* [[#1769]]: sqlx-cli: add `--source` to migration subcommands [[@pedromfedricci]]
* [[#1774]]: Postgres: make `extra_float_digits` settable [[@abonander]]
    * Can be set to `None` for Postgres or third-party database servers that don't support the option.
* [[#1776]]: Implement close-event notification for Pool [[@abonander]]
    * Also fixes `PgListener` preventing `Pool::close()` from resolving.
* [[#1780]]: Implement query builder [[@crajcan]]
    * See also [[#1790]]: Document and expand query builder [[@abonander]]
* [[#1781]]: Postgres: support `NUMERIC[]` using `decimal` feature [[@tm-drtina]]
* [[#1784]]: SQLite: add `FromStr`, `Copy`, `PartialEq`, `Eq` impls for options enums [[@andrewwhitehead]]

### Changed
* [[#1625]]: Update RustCrypto crates [[@paolobarbolini]]
* [[#1725]]: Update `heck` to 0.4 [[@paolobarbolini]]
* [[#1738]]: Update `regex` [[@Dylan-DPC]]
* [[#1763]]: SQLite: update `libsqlite3-sys` [[@espindola]]


### Fixed
* [[#1719]]: Fix a link in `query!()` docs [[@vbmade2000]]
* [[#1731]]: Postgres: fix option passing logic [[@liushuyu]]
* [[#1735]]: sqlx-cli: pass `DATABASE_URL` to command spawned in `prepare` [[@LovecraftianHorror]]
* [[#1741]]: Postgres: fix typo in `TSTZRANGE` [[@mgrachev]]
* [[#1761]]: Fix link from `QueryAs` to `query_as()` in docs [[@mgrachev]]
* [[#1786]]: MySQL: silence compile warnings for unused fields [[@andrewwhitehead]]
* [[#1789]]: SQLite: fix left-joins breaking `query!()` macros [[@tyrelr]]
* [[#1791]]: Postgres: fix newline parsing of `.pgpass` files [[@SebastienGllmt]]
* [[#1799]]: `PoolConnection`: don't leak connection permit if drop task fails to run [[@abonander]]

[#1625]: https://github.com/launchbadge/sqlx/pull/1625
[#1641]: https://github.com/launchbadge/sqlx/pull/1641
[#1675]: https://github.com/launchbadge/sqlx/pull/1675
[#1719]: https://github.com/launchbadge/sqlx/pull/1719
[#1722]: https://github.com/launchbadge/sqlx/pull/1722
[#1725]: https://github.com/launchbadge/sqlx/pull/1725
[#1731]: https://github.com/launchbadge/sqlx/pull/1731
[#1735]: https://github.com/launchbadge/sqlx/pull/1735
[#1736]: https://github.com/launchbadge/sqlx/pull/1736
[#1738]: https://github.com/launchbadge/sqlx/pull/1738
[#1741]: https://github.com/launchbadge/sqlx/pull/1741
[#1748]: https://github.com/launchbadge/sqlx/pull/1748
[#1754]: https://github.com/launchbadge/sqlx/pull/1754
[#1757]: https://github.com/launchbadge/sqlx/pull/1757
[#1761]: https://github.com/launchbadge/sqlx/pull/1761
[#1763]: https://github.com/launchbadge/sqlx/pull/1763
[#1769]: https://github.com/launchbadge/sqlx/pull/1769
[#1774]: https://github.com/launchbadge/sqlx/pull/1774
[#1776]: https://github.com/launchbadge/sqlx/pull/1776
[#1780]: https://github.com/launchbadge/sqlx/pull/1780
[#1781]: https://github.com/launchbadge/sqlx/pull/1781
[#1784]: https://github.com/launchbadge/sqlx/pull/1784
[#1786]: https://github.com/launchbadge/sqlx/pull/1786
[#1789]: https://github.com/launchbadge/sqlx/pull/1789
[#1790]: https://github.com/launchbadge/sqlx/pull/1790
[#1791]: https://github.com/launchbadge/sqlx/pull/1791
[#1799]: https://github.com/launchbadge/sqlx/pull/1799

[0.5.12-prs]: https://github.com/launchbadge/sqlx/pulls?q=is%3Apr+is%3Amerged+merged%3A2022-02-19..2022-04-13

## 0.5.11 - 2022-02-17
[20 pull requests][0.5.11-prs] were merged this release cycle.

### Added
* [[#1610]]: Allow converting `AnyConnectOptions` to a specific `ConnectOptions` [[@05storm26]]
* [[#1652]]: Implement `From` for `AnyConnection` [[@genusistimelord]]
* [[#1658]]: Handle `SQLITE_LOCKED` [[@madadam]]
* [[#1665]]: Document offline mode usage with feature flags [[@sedrik]]
* [[#1680]]: Show checksum mismatches in `sqlx migrate info` [[@ifn3]]
* [[#1685]]: Add tip for setting `opt-level` for `sqlx-macros` [[@LovecraftianHorror]]
* [[#1687]]: Docs: `Acquire` examples and alternative [[@stoically]]
* [[#1696]]: Postgres: support for `ltree` [[@cemoktra]]
* [[#1710]]: Postgres: support for `lquery` [[@cemoktra]]

### Changed
* [[#1605]]: Remove unused dependencies [[@paolobarbolini]]
* [[#1606]]: Add target context to Postgres `NOTICE` logs [[@dbeckwith]]
* [[#1684]]: Macros: Cache parsed `sqlx-data.json` instead of reparsing [[@LovecraftianHorror]]

### Fixed
* [[#1608]]: Drop worker shared state in shutdown (SQLite) [[@andrewwhitehead]]
* [[#1619]]: Docs(macros): remove sentences banning usage of `as _` [[@k-jun]]
* [[#1626]]: Simplify `cargo-sqlx` command-line definition [[@tranzystorek-io]]
* [[#1636]]: Fix and extend Postgres transaction example [[@taladar]]
* [[#1657]]: Fix typo in macro docs [[@p9s]]
* [[#1661]]: Fix binding `Option<T>` for `Any` driver [[@ArGGu]]
* [[#1667]]: MySQL: Avoid panicking if packet is empty [[@nappa85]]
* [[#1692]]: Postgres: Fix power calculation when encoding `BigDecimal` into `NUMERIC` [[@VersBinarii]]

Additionally, we have introduced two mitigations for [the issue of the cyclic dependency on `ahash`][aHash#95]: 

* We re-downgraded our version requirement on `indexmap` from `1.7.0` back to `1.6.2` so users can pin it to that
  version [as recommended in aHash#95][ahash-fix]. 
  * [This was regressed accidentally during a sweeping dependency upgrade before the last release][indexmap-regression],
    sorry about that.
* Thanks to the work of [@LovecraftianHorror] in [#1684], we no longer require the `preserve_order` feature of
  `serde_json` which gives users another place to break the cycle by simply not enabling that feature. 
  * This may introduce extra churn in Git diffs for `sqlx-data.json`, however. If this is an issue for you but 
    the dependency cycle isn't, you can re-enable the `preserve_order` feature:
  ```toml
  [dependencies]
  serde_json = { version = "1", features = ["preserve_order"] }
  ```

[aHash#95]: https://github.com/tkaitchuck/aHash/issues/95
[ahash-fix]: https://github.com/tkaitchuck/aHash/issues/95#issuecomment-874150078
[indexmap-regression]: https://github.com/launchbadge/sqlx/pull/1603#issuecomment-1010827637

[#1605]: https://github.com/launchbadge/sqlx/pull/1605
[#1606]: https://github.com/launchbadge/sqlx/pull/1606
[#1608]: https://github.com/launchbadge/sqlx/pull/1608
[#1610]: https://github.com/launchbadge/sqlx/pull/1610
[#1619]: https://github.com/launchbadge/sqlx/pull/1619
[#1626]: https://github.com/launchbadge/sqlx/pull/1626
[#1636]: https://github.com/launchbadge/sqlx/pull/1636
[#1652]: https://github.com/launchbadge/sqlx/pull/1652
[#1657]: https://github.com/launchbadge/sqlx/pull/1657
[#1658]: https://github.com/launchbadge/sqlx/pull/1658
[#1661]: https://github.com/launchbadge/sqlx/pull/1661
[#1665]: https://github.com/launchbadge/sqlx/pull/1665
[#1667]: https://github.com/launchbadge/sqlx/pull/1667
[#1680]: https://github.com/launchbadge/sqlx/pull/1680
[#1684]: https://github.com/launchbadge/sqlx/pull/1684
[#1685]: https://github.com/launchbadge/sqlx/pull/1685
[#1687]: https://github.com/launchbadge/sqlx/pull/1687
[#1692]: https://github.com/launchbadge/sqlx/pull/1692
[#1696]: https://github.com/launchbadge/sqlx/pull/1696
[#1710]: https://github.com/launchbadge/sqlx/pull/1710

[0.5.11-prs]: https://github.com/launchbadge/sqlx/pulls?q=is%3Apr+is%3Amerged+merged%3A2021-12-30..2022-02-17

## 0.5.10 - 2021-12-29
[A whopping 31 pull requests][0.5.10-prs] were merged this release cycle!

According to this changelog, we saw 18 new contributors! However, some of these folks may have missed getting
mentioned in previous entries since we only listed highlights. To avoid anyone feeling left out, I put in the effort
this time and tried to list every single one here.

### Added
* [[#1228]]: Add `Pool::any_kind()` [[@nitnelave]]
* [[#1343]]: Add `Encode/Decode` impl for `Cow<'_, str>` [[@Drevoed]]
* [[#1474]]: Derive `Clone`, `Copy` for `AnyKind` [[@yuyawk]]
* [[#1497]]: Update FAQ to explain how to configure docs.rs to build a project using SQLx [[@russweas]]
* [[#1498]]: Add description of migration file structure to `migrate!()` docs [[@zbigniewzolnierowicz]]
* [[#1508]]: Add `.persistent(bool)` to `QueryAs`, `QueryScalar` [[@akiradeveloper]]
* [[#1514]]: Add support for serialized threading mode to SQLite [[@LLBlumire]]
* [[#1523]]: Allow `rust_decimal::Decimal` in `PgRange` [[@meh]]
* [[#1539]]: Support `PGOPTIONS` and adding custom configuration options in `PgConnectOptions` [[@liushuyu]]
* [[#1562]]: Re-export `either::Either` used by `Executor::fetch_many()` [[@DoumanAsh]]
* [[#1584]]: Add feature to use RusTLS instead of `native-tls` for `sqlx-cli` [[@SonicZentropy]]
* [[#1592]]: Add `AnyConnection::kind()` [[@05storm26]]

### Changes
* [[#1385]]: Rewrite Postgres array handling to reduce boilerplate and allow custom types [[@jplatte]]
* [[#1479]]: Remove outdated mention of `runtime-async-std-native-tls` as the default runtime in README.md [[@yerke]]
* [[#1526]]: Revise `Pool` docs in a couple places [[@abonander]]
* [[#1535]]: Bump `libsqlite-sys` to `0.23.1` [[@nitsky]]
* [[#1551]]: SQLite: make worker thread responsible for all FFI calls [[@abonander]]
    * If you were encountering segfaults with the SQLite driver, there's a good chance this will fix it!
* [[#1557]]: CI: test with Postgres 14 [[@paolobarbolini]]
* [[#1571]]: Make `whoami` dep optional, only pull it in for Postgres [[@joshtriplett]]
* [[#1572]]: Update `rsa` crate to 0.5 [[@paolobarbolini]]
* [[#1591]]: List SeaORM as an ORM option in the README [[@kunjee17]]
* [[#1601]]: Update `itoa` and `dirs` [[@paolobarbolini]]

### Fixes
* [[#1475]]: Fix panic when converting a negative `chrono::Duration` to `PgInterval` [[@yuyawk]]
* [[#1483]]: Fix error when decoding array of custom types from Postgres [[@demurgos]
* [[#1501]]: Reduce `indexmap` version requirement to `1.6.2` [[@dimfeld]]
* [[#1511]]: Fix element type given to Postgres for arrays of custom enums [[@chesedo]]
* [[#1517]]: Fix mismatched type errors in MySQL type tests [[@abonander]]
* [[#1537]]: Fix missing re-export of `PgCopyIn` [[@akiradeveloper]]
* [[#1566]]: Match `~/.pgpass` password after URL parsing and fix user and database ordering [[@D1plo1d]]
* [[#1582]]: `cargo sqlx prepare`: Append to existing `RUSTFLAGS` instead of overwriting [[@tkintscher]]
* [[#1587]]: SQLite: if set, send `PRAGMA key` on a new connection before anything else. [[@parazyd]]
    * This should fix problems with being unable to open databases using SQLCipher.
    

[#1228]: https://github.com/launchbadge/sqlx/pull/1228
[#1343]: https://github.com/launchbadge/sqlx/pull/1343
[#1385]: https://github.com/launchbadge/sqlx/pull/1385
[#1474]: https://github.com/launchbadge/sqlx/pull/1474
[#1475]: https://github.com/launchbadge/sqlx/pull/1475
[#1479]: https://github.com/launchbadge/sqlx/pull/1479
[#1483]: https://github.com/launchbadge/sqlx/pull/1483
[#1497]: https://github.com/launchbadge/sqlx/pull/1497
[#1498]: https://github.com/launchbadge/sqlx/pull/1498
[#1501]: https://github.com/launchbadge/sqlx/pull/1501
[#1508]: https://github.com/launchbadge/sqlx/pull/1508 
[#1511]: https://github.com/launchbadge/sqlx/pull/1511
[#1514]: https://github.com/launchbadge/sqlx/pull/1514
[#1517]: https://github.com/launchbadge/sqlx/pull/1517
[#1523]: https://github.com/launchbadge/sqlx/pull/1523
[#1526]: https://github.com/launchbadge/sqlx/pull/1526
[#1535]: https://github.com/launchbadge/sqlx/pull/1535
[#1537]: https://github.com/launchbadge/sqlx/pull/1537
[#1539]: https://github.com/launchbadge/sqlx/pull/1539
[#1551]: https://github.com/launchbadge/sqlx/pull/1551
[#1557]: https://github.com/launchbadge/sqlx/pull/1557
[#1562]: https://github.com/launchbadge/sqlx/pull/1562
[#1566]: https://github.com/launchbadge/sqlx/pull/1566
[#1571]: https://github.com/launchbadge/sqlx/pull/1571
[#1572]: https://github.com/launchbadge/sqlx/pull/1572
[#1582]: https://github.com/launchbadge/sqlx/pull/1582
[#1584]: https://github.com/launchbadge/sqlx/pull/1584
[#1587]: https://github.com/launchbadge/sqlx/pull/1587
[#1591]: https://github.com/launchbadge/sqlx/pull/1591
[#1592]: https://github.com/launchbadge/sqlx/pull/1592
[#1601]: https://github.com/launchbadge/sqlx/pull/1601
[0.5.10-prs]: https://github.com/launchbadge/sqlx/pulls?page=1&q=is%3Apr+merged%3A2021-10-02..2021-12-31+sort%3Acreated-asc

## 0.5.9 - 2021-10-01

A hotfix release to address the issue of the `sqlx` crate itself still depending on older versions of `sqlx-core` and 
`sqlx-macros`.

No other changes from `0.5.8`.

## 0.5.8 - 2021-10-01 (Yanked; use 0.5.9)

[A total of 24 pull requests][0.5.8-prs] were merged this release cycle! Some highlights: 

* [[#1289]] Support the `immutable` option on SQLite connections [[@djmarcin]]
* [[#1295]] Support custom initial options for SQLite [[@ghassmo]]
    * Allows specifying custom `PRAGMA`s and overriding those set by SQLx.
* [[#1345]] Initial support for Postgres `COPY FROM/TO`[[@montanalow], [@abonander]]
* [[#1439]] Handle multiple waiting results correctly in MySQL [[@eagletmt]]

[#1289]: https://github.com/launchbadge/sqlx/pull/1289
[#1295]: https://github.com/launchbadge/sqlx/pull/1295
[#1345]: https://github.com/launchbadge/sqlx/pull/1345
[#1439]: https://github.com/launchbadge/sqlx/pull/1439
[0.5.8-prs]: https://github.com/launchbadge/sqlx/pulls?q=is%3Apr+is%3Amerged+merged%3A2021-08-21..2021-10-01

## 0.5.7 - 2021-08-20

* [[#1392]] use `resolve_path` when getting path for `include_str!()` [[@abonander]]
    * Fixes a regression introduced by [[#1332]].
* [[#1393]] avoid recursively spawning tasks in `PgListener::drop()` [[@abonander]]
    * Fixes a panic that occurs when `PgListener` is dropped in `async fn main()`.

[#1392]: https://github.com/launchbadge/sqlx/pull/1392
[#1393]: https://github.com/launchbadge/sqlx/pull/1393

## 0.5.6 - 2021-08-16

A large bugfix release, including but not limited to:

* [[#1329]] Implement `MACADDR` type for Postgres [[@nomick]]
* [[#1363]] Fix `PortalSuspended` for array of composite types in Postgres [[@AtkinsChang]]
* [[#1320]] Reimplement `sqlx::Pool` internals using `futures-intrusive` [[@abonander]]
    * This addresses a number of deadlocks/stalls on acquiring connections from the pool.
* [[#1332]] Macros: tell the compiler about external files/env vars to watch [[@abonander]]
    * Includes `sqlx build-script` to create a `build.rs` to watch `migrations/` for changes.
    * Nightly users can try `RUSTFLAGS=--cfg sqlx_macros_unstable` to tell the compiler 
      to watch `migrations/` for changes instead of using a build script. 
    * See the new section in the docs for `sqlx::migrate!()` for details.
* [[#1351]] Fix a few sources of segfaults/errors in SQLite driver [[@abonander]]
    * Includes contributions from [[@link2ext]] and [[@madadam]].
* [[#1323]] Keep track of column typing in SQLite EXPLAIN parsing [[@marshoepial]]
    * This fixes errors in the macros when using `INSERT/UPDATE/DELETE ... RETURNING ...` in SQLite.
    
[A total of 25 pull requests][0.5.6-prs] were merged this release cycle!

[#1329]: https://github.com/launchbadge/sqlx/pull/1329
[#1363]: https://github.com/launchbadge/sqlx/pull/1363
[#1320]: https://github.com/launchbadge/sqlx/pull/1320
[#1332]: https://github.com/launchbadge/sqlx/pull/1332
[#1351]: https://github.com/launchbadge/sqlx/pull/1351
[#1323]: https://github.com/launchbadge/sqlx/pull/1323
[0.5.6-prs]: https://github.com/launchbadge/sqlx/pulls?q=is%3Apr+is%3Amerged+merged%3A2021-05-24..2021-08-17

## 0.5.5 - 2021-05-24

-   [[#1242]] Fix infinite loop at compile time when using query macros [[@toshokan]]

[#1242]: https://github.com/launchbadge/sqlx/pull/1242

## 0.5.4 - 2021-05-22

-   [[#1235]] Fix compilation with rustls from an eager update to webpki [[@ETCaton]]

[#1235]: https://github.com/launchbadge/sqlx/pull/1235

## 0.5.3 - 2021-05-21

-   [[#1211]] Even more tweaks and fixes to the Pool internals [[@abonander]]

-   [[#1213]] Add support for bytes and `chrono::NaiveDateTime` to `Any` [[@guylapid]]

-   [[#1224]] Add support for `chrono::DateTime<Local>` to `Any` with `MySQL` [[@NatPRoach]]

-   [[#1216]] Skip empty lines and comments in pgpass files [[@feikesteenbergen]]

-   [[#1218]] Add support for `PgMoney` to the compile-time type-checking [[@iamsiddhant05]]

[#1211]: https://github.com/launchbadge/sqlx/pull/1211
[#1213]: https://github.com/launchbadge/sqlx/pull/1213
[#1216]: https://github.com/launchbadge/sqlx/pull/1216
[#1218]: https://github.com/launchbadge/sqlx/pull/1218
[#1224]: https://github.com/launchbadge/sqlx/pull/1224

## 0.5.2 - 2021-04-15

-   [[#1149]] Tweak and optimize Pool internals [[@abonander]]

-   [[#1132]] Remove `'static` bound on `Connection::transaction` [[@argv-minus-one]]

-   [[#1128]] Fix `-y` flag for `sqlx db reset -y` [[@qqwa]]

-   [[#1099]] [[#1097]] Truncate buffer when `BufStream` is dropped [[@Diggsey]]

[#1132]: https://github.com/launchbadge/sqlx/pull/1132
[#1149]: https://github.com/launchbadge/sqlx/pull/1149
[#1128]: https://github.com/launchbadge/sqlx/pull/1128
[#1099]: https://github.com/launchbadge/sqlx/pull/1099
[#1097]: https://github.com/launchbadge/sqlx/issues/1097

### PostgreSQL

-   [[#1170]] Remove `Self: Type` bounds in `Encode` / `Decode` implementations for arrays [[@jplatte]]

    Enables working around the lack of support for user-defined array types:

    ```rust
    #[derive(sqlx::Encode)]
    struct Foos<'a>(&'a [Foo]);

    impl sqlx::Type<sqlx::Postgres> for Foos<'_> {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_name("_foo")
        }
    }

    query_as!(
        Whatever,
        "<QUERY with $1 of type foo[]>",
        Foos(&foo_vec) as _,
    )
    ```

-   [[#1141]] Use `u16::MAX` instead of `i16::MAX` for a check against the largest number of parameters in a query [[@crajcan]]

-   [[#1112]] Add support for `DOMAIN` types [[@demurgos]]

-   [[#1100]] Explicitly `UNLISTEN` before returning connections to the pool in `PgListener` [[@Diggsey]]

[#1170]: https://github.com/launchbadge/sqlx/pull/1170
[#1141]: https://github.com/launchbadge/sqlx/pull/1141
[#1112]: https://github.com/launchbadge/sqlx/pull/1112
[#1100]: https://github.com/launchbadge/sqlx/pull/1100

### SQLite

-   [[#1161]] Catch `SQLITE_MISUSE` on connection close and panic [[@link2xt]]

-   [[#1160]] Do not cast pointers to `i32` (cast to `usize`) [[@link2xt]]

-   [[#1156]] Reset the statement when `fetch_many` stream is dropped [[@link2xt]]

[#1161]: https://github.com/launchbadge/sqlx/pull/1161
[#1160]: https://github.com/launchbadge/sqlx/pull/1160
[#1156]: https://github.com/launchbadge/sqlx/pull/1156

## 0.5.1 - 2021-02-04

-   Update sqlx-rt to 0.3.

## 0.5.0 - 2021-02-04

### Changes

-   [[#983]] [[#1022]] Upgrade async runtime dependencies [[@seryl], [@ant32], [@jplatte], [@robjtede]]

    -   tokio 1.0
    -   actix-rt 2.0

-   [[#854]] Allow chaining `map` and `try_map` [[@jplatte]]

    Additionally enables calling these combinators with the macros:

    ```rust
    let ones: Vec<i32> = query!("SELECT 1 as foo")
        .map(|row| row.foo)
        .fetch_all(&mut conn).await?;
    ```

-   [[#940]] Rename the `#[sqlx(rename)]` attribute used to specify the type name on the database
    side to `#[sqlx(type_name)]` [[@jplatte]].

-   [[#976]] Rename the `DbDone` types to `DbQueryResult`. [[@jplatte]]

-   [[#976]] Remove the `Done` trait. The `.rows_affected()` method is now available as an inherent
    method on `PgQueryResult`, `MySqlQueryResult` and so on. [[@jplatte]]

-   [[#1007]] Remove `any::AnyType` (and replace with directly implementing `Type<Any>`) [[@jplatte]]

### Added

-   [[#998]] [[#821]] Add `.constraint()` to `DatabaseError` [[@fl9]]

-   [[#919]] For SQLite, add support for unsigned integers [[@dignifiedquire]]

### Fixes

-   [[#1002]] For SQLite, `GROUP BY` in `query!` caused an infinite loop at compile time. [[@pymongo]]

-   [[#979]] For MySQL, fix support for non-default authentication. [[@sile]]

-   [[#918]] Recover from dropping `wait_for_conn` inside Pool. [[@antialize]]

[#821]: https://github.com/launchbadge/sqlx/issues/821
[#918]: https://github.com/launchbadge/sqlx/pull/918
[#919]: https://github.com/launchbadge/sqlx/pull/919
[#983]: https://github.com/launchbadge/sqlx/pull/983
[#940]: https://github.com/launchbadge/sqlx/pull/940
[#976]: https://github.com/launchbadge/sqlx/pull/976
[#979]: https://github.com/launchbadge/sqlx/pull/979
[#998]: https://github.com/launchbadge/sqlx/pull/998
[#983]: https://github.com/launchbadge/sqlx/pull/983
[#1002]: https://github.com/launchbadge/sqlx/pull/1002
[#1007]: https://github.com/launchbadge/sqlx/pull/1007
[#1022]: https://github.com/launchbadge/sqlx/pull/1022

## 0.4.2 - 2020-12-19

-   [[#908]] Fix `whoami` crash on FreeBSD platform [[@fundon]] [[@AldaronLau]]

-   [[#895]] Decrement pool size when connection is released [[@andrewwhitehead]]

-   [[#878]] Fix `conn.transaction` wrapper [[@hamza1311]]

    ```rust
    conn.transaction(|transaction: &mut Transaction<Database> | {
        // ...
    });
    ```

-   [[#874]] Recognize `1` as `true` for `SQLX_OFFLINE [[@Pleto]]

-   [[#747]] [[#867]] Replace `lru-cache` with `hashlink` [[@chertov]]

-   [[#860]] Add `rename_all` to `FromRow` and add `camelCase` and `PascalCase` [[@framp]]

-   [[#839]] Add (optional) support for `bstr::BStr`, `bstr::BString`, and `git2::Oid` [[@joshtriplett]]

#### SQLite

-   [[#893]] Fix memory leak if `create_collation` fails [[@slumber]]

-   [[#852]] Fix potential 100% CPU usage in `fetch_one` / `fetch_optional` [[@markazmierczak]]

-   [[#850]] Add `synchronous` option to `SqliteConnectOptions` [[@markazmierczak]]

#### PostgreSQL

-   [[#889]] Fix decimals (one more time) [[@slumber]]

-   [[#876]] Add support for `BYTEA[]` to compile-time type-checking [[@augustocdias]]

-   [[#845]] Fix path for `&[NaiveTime]` in `query!` macros [[@msrd0]]

#### MySQL

-   [[#880]] Consider `utf8mb4_general_ci` as a string [[@mcronce]]

[#908]: https://github.com/launchbadge/sqlx/pull/908
[#895]: https://github.com/launchbadge/sqlx/pull/895
[#893]: https://github.com/launchbadge/sqlx/pull/893
[#889]: https://github.com/launchbadge/sqlx/pull/889
[#880]: https://github.com/launchbadge/sqlx/pull/880
[#878]: https://github.com/launchbadge/sqlx/pull/878
[#876]: https://github.com/launchbadge/sqlx/pull/876
[#874]: https://github.com/launchbadge/sqlx/pull/874
[#867]: https://github.com/launchbadge/sqlx/pull/867
[#860]: https://github.com/launchbadge/sqlx/pull/860
[#854]: https://github.com/launchbadge/sqlx/pull/854
[#852]: https://github.com/launchbadge/sqlx/pull/852
[#850]: https://github.com/launchbadge/sqlx/pull/850
[#845]: https://github.com/launchbadge/sqlx/pull/845
[#839]: https://github.com/launchbadge/sqlx/pull/839
[#747]: https://github.com/launchbadge/sqlx/issues/747

## 0.4.1 – 2020-11-13

Fix docs.rs build by enabling a runtime feature in the docs.rs metadata in `Cargo.toml`.

## 0.4.0 - 2020-11-12

-   [[#774]] Fix usage of SQLx derives with other derive crates [[@NyxCode]]

-   [[#762]] Fix `migrate!()` (with no params) [[@esemeniuc]]

-   [[#755]] Add `kebab-case` to `rename_all` [[@iamsiddhant05]]

-   [[#735]] Support `rustls` [[@jplatte]]

    Adds `-native-tls` or `-rustls` on each runtime feature:

    ```toml
    # previous
    features = [ "runtime-async-std" ]

    # now
    features = [ "runtime-async-std-native-tls" ]
    ```

-   [[#718]] Support tuple structs with `#[derive(FromRow)]` [[@dvermd]]

#### SQLite

-   [[#789]] Support `$NNN` parameters [[@nitsky]]

-   [[#784]] Use `futures_channel::oneshot` in worker for big perf win [[@markazmierczak]]

#### PostgreSQL

-   [[#781]] Fix decimal conversions handling of `0.01` [[@pimeys]]

-   [[#745]] Always prefer parsing of the non-localized notice severity field [[@dstoeckel]]

-   [[#742]] Enable `Vec<DateTime<Utc>>` with chrono [[@mrcd]]

#### MySQL

-   [[#743]] Consider `utf8mb4_bin` as a string [[@digorithm]]

-   [[#739]] Fix minor protocol detail with `iteration-count` that was blocking Vitess [[@mcronce]]

[#774]: https://github.com/launchbadge/sqlx/pull/774
[#789]: https://github.com/launchbadge/sqlx/pull/789
[#784]: https://github.com/launchbadge/sqlx/pull/784
[#781]: https://github.com/launchbadge/sqlx/pull/781
[#762]: https://github.com/launchbadge/sqlx/pull/762
[#755]: https://github.com/launchbadge/sqlx/pull/755
[#745]: https://github.com/launchbadge/sqlx/pull/745
[#743]: https://github.com/launchbadge/sqlx/pull/743
[#742]: https://github.com/launchbadge/sqlx/pull/742
[#735]: https://github.com/launchbadge/sqlx/pull/735
[#739]: https://github.com/launchbadge/sqlx/pull/739
[#718]: https://github.com/launchbadge/sqlx/pull/718

## 0.4.0-beta.1 - 2020-07-27

### Highlights

-   Enable compile-time type checking from cached metadata to enable building
    in an environment without access to a development database (e.g., Docker, CI).

-   Initial support for **Microsoft SQL Server**. If there is something missing that you need,
    open an issue. We are happy to help.

-   SQL migrations, both with a CLI tool and programmatically loading migrations at runtime.

-   Runtime-determined database driver, `Any`, to support compile-once and run with a database
    driver selected at runtime.

-   Support for user-defined types and more generally overriding the inferred Rust type from SQL
    with compile-time SQL verification.

### Fixed

#### MySQL

-   [[#418]] Support zero dates and times [[@blackwolf12333]]

### Added

-   [[#174]] Inroduce a builder to construct connections to bypass the URL parsing

    ```rust
    // MSSQL
    let conn = MssqlConnectOptions::new()
        .host("localhost")
        .database("master")
        .username("sa")
        .password("Password")
        .connect().await?;

    // SQLite
    let conn = SqliteConnectOptions::from_str("sqlite://a.db")?
        .foreign_keys(false)
        .connect().await?;
    ```

-   [[#127]] Get the last ID or Row ID inserted for MySQL or SQLite

    ```rust
    // MySQL
    let id: u64 = query!("INSERT INTO table ( col ) VALUES ( ? )", val)
        .execute(&mut conn).await?
        .last_insert_id(); // LAST_INSERT_ID()

    // SQLite
    let id: i64 = query!("INSERT INTO table ( col ) VALUES ( ?1 )", val)
        .execute(&mut conn).await?
        .last_insert_rowid(); // sqlite3_last_insert_rowid()
    ```

-   [[#263]] Add hooks to the Pool: `after_connect`, `before_release`, and `after_acquire`

    ```rust
    // PostgreSQL
    let pool = PgPoolOptions::new()
        .after_connect(|conn| Box::pin(async move {
            conn.execute("SET application_name = 'your_app';").await?;
            conn.execute("SET search_path = 'my_schema';").await?;

            Ok(())
        }))
        .connect("postgres:// …").await?
    ```

-   [[#308]] [[#495]] Extend `derive(FromRow)` with support for `#[sqlx(default)]` on fields to allow reading in a partial query [[@OriolMunoz]]

-   [[#454]] [[#456]] Support `rust_decimal::Decimal` as an alternative to `bigdecimal::BigDecimal` for `NUMERIC` columns in MySQL and PostgreSQL [[@pimeys]]

-   [[#181]] Column names and type information is now accessible from `Row` via `Row::columns()` or `Row::column(name)`

#### PostgreSQL

-   [[#197]] [[#271]] Add initial support for `INTERVAL` (full support pending a `time::Period` type) [[@dimtion]]

#### MySQL

-   [[#449]] [[#450]] Support Unix Domain Sockets (UDS) for MySQL [[@pimeys]]

#### SQLite

-   Types are now inferred for expressions. This means its now possible to use `query!` and `query_as!` for:

    ```rust
    let row = query!("SELECT 10 as _1, x + 5 as _2 FROM table").fetch_one(&mut conn).await?;

    assert_eq!(row._1, 10);
    assert_eq!(row._2, 5); // 5 + x?
    ```

-   [[#167]] Support `foreign_keys` explicitly with a `foreign_keys(true)` method available on `SqliteConnectOptions` which is a builder
    for new SQLite connections (and can be passed into `PoolOptions` to build a pool).

    ```rust
    let conn = SqliteConnectOptions::new()
        .foreign_keys(true) // on by default
        .connect().await?;
    ```

-   [[#430]] [[#438]] Add method to get the raw SQLite connection handle [[@agentsim]]

    ```rust
    // conn is `SqliteConnection`
    // this is not unsafe, but what you do with the handle will be
    let ptr: *mut libsqlite3::sqlite3 = conn.as_raw_handle();
    ```

-   [[#164]] Support `TIMESTAMP`, `DATETIME`, `DATE`, and `TIME` via `chrono` in SQLite [[@felipesere]] [[@meteficha]]

### Changed

-   `Transaction` now mutably borrows a connection instead of owning it. This enables a new (or nested) transaction to be started from `&mut conn`.

-   [[#145]] [[#444]] Use a least-recently-used (LRU) cache to limit the growth of the prepared statement cache for SQLite, MySQL, and PostgreSQL [[@pimeys]]

#### SQLite

-   [[#499]] `INTEGER` now resolves to `i64` instead of `i32`, `INT4` will still resolve to `i32`

### Removed

[#127]: https://github.com/launchbadge/sqlx/issues/127
[#174]: https://github.com/launchbadge/sqlx/issues/174
[#145]: https://github.com/launchbadge/sqlx/issues/145
[#164]: https://github.com/launchbadge/sqlx/issues/164
[#167]: https://github.com/launchbadge/sqlx/issues/167
[#181]: https://github.com/launchbadge/sqlx/issues/181
[#197]: https://github.com/launchbadge/sqlx/issues/197
[#263]: https://github.com/launchbadge/sqlx/issues/263
[#308]: https://github.com/launchbadge/sqlx/issues/308
[#418]: https://github.com/launchbadge/sqlx/issues/418
[#430]: https://github.com/launchbadge/sqlx/issues/430
[#449]: https://github.com/launchbadge/sqlx/issues/449
[#499]: https://github.com/launchbadge/sqlx/issues/499
[#454]: https://github.com/launchbadge/sqlx/issues/454
[#271]: https://github.com/launchbadge/sqlx/pull/271
[#444]: https://github.com/launchbadge/sqlx/pull/444
[#438]: https://github.com/launchbadge/sqlx/pull/438
[#495]: https://github.com/launchbadge/sqlx/pull/495
[#495]: https://github.com/launchbadge/sqlx/pull/495

## 0.3.5 - 2020-05-06

### Fixed

-   [[#259]] Handle percent-encoded paths for SQLite [[@g-s-k]]

-   [[#281]] Deallocate SQLite statements before closing the SQLite connection [[@hasali19]]

-   [[#284]] Fix handling of `0` for `BigDecimal` in PostgreSQL and MySQL [[@abonander]]

### Added

-   [[#256]] Add `query_unchecked!` and `query_file_unchecked!` with similar semantics to `query_as_unchecked!` [[@meh]]

-   [[#252]] [[#297]] Derive several traits for the `Json<T>` wrapper type [[@meh]]

-   [[#261]] Add support for `#[sqlx(rename_all = "snake_case")]` to `#[derive(Type)]` [[@shssoichiro]]

-   [[#253]] Add support for UNIX domain sockets to PostgreSQL [[@Nilix007]]

-   [[#251]] Add support for textual JSON on MySQL [[@blackwolf12333]]

-   [[#275]] [[#268]] Optionally log formatted SQL queries on execution [[@shssoichiro]]

-   [[#267]] Support Cargo.toml relative `.env` files; allows for each crate in a workspace to use their own `.env` file and thus their own `DATABASE_URL` [[@xyzd]]

[#252]: https://github.com/launchbadge/sqlx/pull/252
[#261]: https://github.com/launchbadge/sqlx/pull/261
[#256]: https://github.com/launchbadge/sqlx/pull/256
[#259]: https://github.com/launchbadge/sqlx/pull/259
[#253]: https://github.com/launchbadge/sqlx/pull/253
[#297]: https://github.com/launchbadge/sqlx/pull/297
[#251]: https://github.com/launchbadge/sqlx/pull/251
[#275]: https://github.com/launchbadge/sqlx/pull/275
[#267]: https://github.com/launchbadge/sqlx/pull/267
[#268]: https://github.com/launchbadge/sqlx/pull/268
[#281]: https://github.com/launchbadge/sqlx/pull/281
[#284]: https://github.com/launchbadge/sqlx/pull/284

## 0.3.4 - 2020-04-10

### Fixed

-   [[#241]] Type name for custom enum is not always attached to TypeInfo in PostgreSQL

-   [[#237]] [[#238]] User-defined type name matching is now case-insensitive in PostgreSQL [[@qtbeee]]

-   [[#231]] Handle empty queries (and those with comments) in SQLite

-   [[#228]] Provide `MapRow` implementations for functions (enables `.map(|row| ...)` over `.try_map(|row| ...)`)

### Added

-   [[#234]] Add support for `NUMERIC` in MySQL with the `bigdecimal` crate [[@xiaopengli89]]

-   [[#227]] Support `#[sqlx(rename = "new_name")]` on struct fields within a `FromRow` derive [[@sidred]]

[#228]: https://github.com/launchbadge/sqlx/issues/228
[#231]: https://github.com/launchbadge/sqlx/issues/231
[#237]: https://github.com/launchbadge/sqlx/issues/237
[#241]: https://github.com/launchbadge/sqlx/issues/241
[#227]: https://github.com/launchbadge/sqlx/pull/227
[#234]: https://github.com/launchbadge/sqlx/pull/234
[#238]: https://github.com/launchbadge/sqlx/pull/238

## 0.3.3 - 2020-04-01

### Fixed

-   [[#214]] Handle percent-encoded usernames in a database URL [[@jamwaffles]]

### Changed

-   [[#216]] Mark `Cursor`, `Query`, `QueryAs`, `query::Map`, and `Transaction` as `#[must_use]` [[@Ace4896]]

-   [[#213]] Remove matches dependency and use matches macro from std [[@nrjais]]

[#216]: https://github.com/launchbadge/sqlx/pull/216
[#214]: https://github.com/launchbadge/sqlx/pull/214
[#213]: https://github.com/launchbadge/sqlx/pull/213

## 0.3.2 - 2020-03-31

### Fixed

-   [[#212]] Removed sneaky `println!` in `MySqlCursor`

[#212]: https://github.com/launchbadge/sqlx/issues/212

## 0.3.1 - 2020-03-30

### Fixed

-   [[#203]] Allow an empty password for MySQL

-   [[#204]] Regression in error reporting for invalid SQL statements on PostgreSQL

-   [[#200]] Fixes the incorrect handling of raw (`r#...`) fields of a struct in the `FromRow` derive [[@sidred]]

[#200]: https://github.com/launchbadge/sqlx/pull/200
[#203]: https://github.com/launchbadge/sqlx/issues/203
[#204]: https://github.com/launchbadge/sqlx/issues/204

## 0.3.0 - 2020-03-29

### Breaking Changes

-   `sqlx::Row` now has a lifetime (`'c`) tied to the database connection. In effect, this means that you cannot store `Row`s or collect
    them into a collection. `Query` (returned from `sqlx::query()`) has `map()` which takes a function to map from the `Row` to
    another type to make this transition easier.

    In 0.2.x

    ```rust
    let rows = sqlx::query("SELECT 1")
        .fetch_all(&mut conn).await?;
    ```

    In 0.3.x

    ```rust
    let values: Vec<i32> = sqlx::query("SELECT 1")
        .map(|row: PgRow| row.get(0))
        .fetch_all(&mut conn).await?;
    ```

    To assist with the above, `sqlx::query_as()` now supports querying directly into tuples (up to 9 elements) or
    struct types with a `#[derive(FromRow)]`.

    ```rust
    // This extension trait is needed until a rust bug is fixed
    use sqlx::postgres::PgQueryAs;

    let values: Vec<(i32, bool)> = sqlx::query_as("SELECT 1, false")
        .fetch_all(&mut conn).await?;
    ```

-   `HasSqlType<T>: Database` is now `T: Type<Database>` to mirror `Encode` and `Decode`

-   `Query::fetch` (returned from `query()`) now returns a new `Cursor` type. `Cursor` is a Stream-like type where the
    item type borrows into the stream (which itself borrows from connection). This means that using `query().fetch()` you can now
    stream directly from the database with **zero-copy** and **zero-allocation**.

-   Remove `PgTypeInfo::with_oid` and replace with `PgTypeInfo::with_name`

### Added

-   Results from the database are now zero-copy and no allocation beyond a shared read buffer
    for the TCP stream ( in other words, almost no per-query allocation ). Bind arguments still
    do allocate a buffer per query.

-   [[#129]] Add support for [SQLite](https://sqlite.org/index.html). Generated code should be very close to normal use of the C API.

    -   Adds `Sqlite`, `SqliteConnection`, `SqlitePool`, and other supporting types

-   [[#97]] [[#134]] Add support for user-defined types. [[@Freax13]]

    -   Rust-only domain types or transparent wrappers around SQL types. These may be used _transparently_ inplace of
        the SQL type.

        ```rust
        #[derive(sqlx::Type)]
        #[repr(transparent)]
        struct Meters(i32);
        ```

    -   Enumerations may be defined in Rust and can match SQL by integer discriminant or variant name.

        ```rust
        #[derive(sqlx::Type)]
        #[repr(i32)] // Expects a INT in SQL
        enum Color { Red = 1, Green = 2, Blue = 3 }
        ```

        ```rust
        #[derive(sqlx::Type)]
        #[sqlx(rename = "TEXT")] // May also be the name of a user defined enum type
        #[sqlx(rename_all = "lowercase")] // similar to serde rename_all
        enum Color { Red, Green, Blue } // expects 'red', 'green', or 'blue'
        ```

    -   **Postgres** further supports user-defined composite types.

        ```rust
        #[derive(sqlx::Type)]
        #[sqlx(rename = "interface_type")]
        struct InterfaceType {
            name: String,
            supplier_id: i32,
            price: f64
        }
        ```

-   [[#98]] [[#131]] Add support for asynchronous notifications in Postgres (`LISTEN` / `NOTIFY`). [[@thedodd]]

    -   Supports automatic reconnection on connection failure.

    -   `PgListener` implements `Executor` and may be used to execute queries. Be careful however as if the
        intent is to handle and process messages rapidly you don't want to be tying up the connection
        for too long. Messages received during queries are buffered and will be delivered on the next call
        to `recv()`.

    ```rust
    let mut listener = PgListener::new(DATABASE_URL).await?;

    listener.listen("topic").await?;

    loop {
        let message = listener.recv().await?;

        println!("payload = {}", message.payload);
    }
    ```

-   Add _unchecked_ variants of the query macros. These will still verify the SQL for syntactic and
    semantic correctness with the current database but they will not check the input or output types.

    This is intended as a temporary solution until `query_as!` is able to support user defined types.

    -   `query_as_unchecked!`
    -   `query_file_as_unchecked!`

-   Add support for many more types in Postgres

    -   `JSON`, `JSONB` [[@oeb25]]
    -   `INET`, `CIDR` [[@PoiScript]]
    -   Arrays [[@oeb25]]
    -   Composites ( Rust tuples or structs with a `#[derive(Type)]` )
    -   `NUMERIC` [[@abonander]]
    -   `OID` (`u32`)
    -   `"CHAR"` (`i8`)
    -   `TIMESTAMP`, `TIMESTAMPTZ`, etc. with the `time` crate [[@utter-step]]
    -   Enumerations ( Rust enums with a `#[derive(Type)]` ) [[@Freax13]]

### Changed

-   `Query` (and `QueryAs`; returned from `query()`, `query_as()`, `query!()`, and `query_as!()`) now will accept both `&mut Connection` or
    `&Pool` where as in 0.2.x they required `&mut &Pool`.

-   `Executor` now takes any value that implements `Execute` as a query. `Execute` is implemented for `Query` and `QueryAs` to mean
    exactly what they've meant so far, a prepared SQL query. However, `Execute` is also implemented for just `&str` which now performs
    a raw or unprepared SQL query. You can further use this to fetch `Row`s from the database though it is not as efficient as the
    prepared API (notably Postgres and MySQL send data back in TEXT mode as opposed to in BINARY mode).

    ```rust
    use sqlx::Executor;

    // Set the time zone parameter
    conn.execute("SET TIME ZONE LOCAL;").await

    // Demonstrate two queries at once with the raw API
    let mut cursor = conn.fetch("SELECT 1; SELECT 2");
    let row = cursor.next().await?.unwrap();
    let value: i32 = row.get(0); // 1
    let row = cursor.next().await?.unwrap();
    let value: i32 = row.get(0); // 2
    ```

### Removed

-   `Query` (returned from `query()`) no longer has `fetch_one`, `fetch_optional`, or `fetch_all`. You _must_ map the row using `map()` and then
    you will have a `query::Map` value that has the former methods available.

    ```rust
    let values: Vec<i32> = sqlx::query("SELECT 1")
        .map(|row: PgRow| row.get(0))
        .fetch_all(&mut conn).await?;
    ```

### Fixed

-   [[#62]] [[#130]] [[#135]] Remove explicit set of `IntervalStyle`. Allow usage of SQLx for CockroachDB and potentially PgBouncer. [[@bmisiak]]

-   [[#108]] Allow nullable and borrowed values to be used as arguments in `query!` and `query_as!`. For example, where the column would
    resolve to `String` in Rust (TEXT, VARCHAR, etc.), you may now use `Option<String>`, `Option<&str>`, or `&str` instead. [[@abonander]]

-   [[#108]] Make unknown type errors far more informative. As an example, trying to `SELECT` a `DATE` column will now try and tell you about the
    `chrono` feature. [[@abonander]]

    ```
    optional feature `chrono` required for type DATE of column #1 ("now")
    ```

[#62]: https://github.com/launchbadge/sqlx/issues/62
[#130]: https://github.com/launchbadge/sqlx/issues/130
[#98]: https://github.com/launchbadge/sqlx/pull/98
[#97]: https://github.com/launchbadge/sqlx/pull/97
[#134]: https://github.com/launchbadge/sqlx/pull/134
[#129]: https://github.com/launchbadge/sqlx/pull/129
[#131]: https://github.com/launchbadge/sqlx/pull/131
[#135]: https://github.com/launchbadge/sqlx/pull/135
[#108]: https://github.com/launchbadge/sqlx/pull/108

## 0.2.6 - 2020-03-10

### Added

-   [[#114]] Export `sqlx_core::Transaction` [[@thedodd]]

### Fixed

-   [[#125]] [[#126]] Fix statement execution in MySQL if it contains NULL statement values [[@repnop]]

-   [[#105]] [[#109]] Allow trailing commas in query macros [[@timmythetiny]]

[#105]: https://github.com/launchbadge/sqlx/pull/105
[#109]: https://github.com/launchbadge/sqlx/pull/109
[#114]: https://github.com/launchbadge/sqlx/pull/114
[#125]: https://github.com/launchbadge/sqlx/pull/125
[#126]: https://github.com/launchbadge/sqlx/pull/126
[@timmythetiny]: https://github.com/timmythetiny
[@thedodd]: https://github.com/thedodd

## 0.2.5 - 2020-02-01

### Fixed

-   Fix decoding of Rows containing NULLs in Postgres [#104]

-   After a large review and some battle testing by [@ianthetechie](https://github.com/ianthetechie)
    of the `Pool`, a live leaking issue was found. This has now been fixed by [@abonander] in [#84] which
    included refactoring to make the pool internals less brittle (using RAII instead of manual
    work is one example) and to help any future contributors when changing the pool internals.

-   Passwords are now being percent-decoded before being presented to the server [[@repnop]]

-   [@100] Fix `FLOAT` and `DOUBLE` decoding in MySQL

[#84]: https://github.com/launchbadge/sqlx/issues/84
[#100]: https://github.com/launchbadge/sqlx/issues/100
[#104]: https://github.com/launchbadge/sqlx/issues/104

### Added

-   [[#72]] Add `PgTypeInfo::with_oid` to allow simple construction of `PgTypeInfo` which enables `HasSqlType`
    to be implemented by downstream consumers of SQLx [[@jplatte]]

-   [[#96]] Add support for returning columns from `query!` with a name of a rust keyword by
    using raw identifiers [[@yaahc]]

-   [[#71]] Implement derives for `Encode` and `Decode`. This is the first step to supporting custom types in SQLx. [[@Freax13]]

[#72]: https://github.com/launchbadge/sqlx/issues/72
[#96]: https://github.com/launchbadge/sqlx/issues/96
[#71]: https://github.com/launchbadge/sqlx/issues/71

## 0.2.4 - 2020-01-18

### Fixed

-   Fix decoding of Rows containing NULLs in MySQL (and add an integration test so this doesn't break again)

## 0.2.3 - 2020-01-18

### Fixed

-   Fix `query!` when used on a query that does not return results

## 0.2.2 - 2020-01-16

### Added

-   [[#57]] Add support for unsigned integers and binary types in `query!` for MySQL [[@mehcode]]

[#57]: https://github.com/launchbadge/sqlx/issues/57

### Fixed

-   Fix stall when requesting TLS from a Postgres server that explicitly does not support TLS (such as postgres running inside docker) [[@abonander]]

-   [[#66]] Declare used features for `tokio` in `sqlx-macros` explicitly

[#66]: https://github.com/launchbadge/sqlx/issues/66

## 0.2.1 - 2020-01-16

### Fixed

-   [[#64], [#65]] Fix decoding of Rows containing NULLs in MySQL [[@danielakhterov]]

[#64]: https://github.com/launchbadge/sqlx/pull/64
[#65]: https://github.com/launchbadge/sqlx/pull/65

-   [[#55]] Use a shared tokio runtime for the `query!` macro compile-time execution (under the `runtime-tokio` feature) [[@udoprog]]

[#55]: https://github.com/launchbadge/sqlx/pull/55

## 0.2.0 - 2020-01-15

### Fixed

-   https://github.com/launchbadge/sqlx/issues/47

### Added

-   Support Tokio through an optional `runtime-tokio` feature.

-   Support SQL transactions. You may now use the `begin()` function on `Pool` or `Connection` to
    start a new SQL transaction. This returns `sqlx::Transaction` which will `ROLLBACK` on `Drop`
    or can be explicitly `COMMIT` using `commit()`.

-   Support TLS connections.

## 0.1.4 - 2020-01-11

### Fixed

-   https://github.com/launchbadge/sqlx/issues/43

-   https://github.com/launchbadge/sqlx/issues/40

### Added

-   Support for `SCRAM-SHA-256` authentication in Postgres [#37](https://github.com/launchbadge/sqlx/pull/37) [@danielakhterov](https://github.com/danielakhterov)

-   Implement `Debug` for Pool [#42](https://github.com/launchbadge/sqlx/pull/42) [@prettynatty](https://github.com/prettynatty)

## 0.1.3 - 2020-01-06

### Fixed

-   https://github.com/launchbadge/sqlx/issues/30

## 0.1.2 - 2020-01-03

### Added

-   Support for Authentication in MySQL 5+ including the newer authentication schemes now default in MySQL 8: `mysql_native_password`, `sha256_password`, and `caching_sha2_password`.

-   [`Chrono`](https://github.com/chronotope/chrono) support for MySQL was only partially implemented (was missing `NaiveTime` and `DateTime<Utc>`).

-   `Vec<u8>` (and `[u8]`) support for MySQL (`BLOB`) and Postgres (`BYTEA`).

[@abonander]: https://github.com/abonander
[@danielakhterov]: https://github.com/danielakhterov
[@mehcode]: https://github.com/mehcode
[@udoprog]: https://github.com/udoprog
[@jplatte]: https://github.com/jplatte
[@yaahc]: https://github.com/yaahc
[@freax13]: https://github.com/Freax13
[@repnop]: https://github.com/repnop
[@bmisiak]: https://github.com/bmisiak
[@oeb25]: https://github.com/oeb25
[@poiscript]: https://github.com/PoiScript
[@utter-step]: https://github.com/utter-step
[@sidred]: https://github.com/sidred
[@ace4896]: https://github.com/Ace4896
[@jamwaffles]: https://github.com/jamwaffles
[@nrjais]: https://github.com/nrjais
[@qtbeee]: https://github.com/qtbeee
[@xiaopengli89]: https://github.com/xiaopengli89
[@meh]: https://github.com/meh
[@shssoichiro]: https://github.com/shssoichiro
[@nilix007]: https://github.com/Nilix007
[@g-s-k]: https://github.com/g-s-k
[@blackwolf12333]: https://github.com/blackwolf12333
[@xyzd]: https://github.com/xyzd
[@hasali19]: https://github.com/hasali19
[@oriolmunoz]: https://github.com/OriolMunoz
[@pimeys]: https://github.com/pimeys
[@agentsim]: https://github.com/agentsim
[@meteficha]: https://github.com/meteficha
[@felipesere]: https://github.com/felipesere
[@dimtion]: https://github.com/dimtion
[@fundon]: https://github.com/fundon
[@aldaronlau]: https://github.com/AldaronLau
[@andrewwhitehead]: https://github.com/andrewwhitehead
[@slumber]: https://github.com/slumber
[@mcronce]: https://github.com/mcronce
[@hamza1311]: https://github.com/hamza1311
[@augustocdias]: https://github.com/augustocdias
[@pleto]: https://github.com/Pleto
[@chertov]: https://github.com/chertov
[@framp]: https://github.com/framp
[@markazmierczak]: https://github.com/markazmierczak
[@msrd0]: https://github.com/msrd0
[@joshtriplett]: https://github.com/joshtriplett
[@nyxcode]: https://github.com/NyxCode
[@nitsky]: https://github.com/nitsky
[@esemeniuc]: https://github.com/esemeniuc
[@iamsiddhant05]: https://github.com/iamsiddhant05
[@dstoeckel]: https://github.com/dstoeckel
[@mrcd]: https://github.com/mrcd
[@dvermd]: https://github.com/dvermd
[@seryl]: https://github.com/seryl
[@ant32]: https://github.com/ant32
[@robjtede]: https://github.com/robjtede
[@pymongo]: https://github.com/pymongo
[@sile]: https://github.com/sile
[@fl9]: https://github.com/fl9
[@antialize]: https://github.com/antialize
[@dignifiedquire]: https://github.com/dignifiedquire
[@argv-minus-one]: https://github.com/argv-minus-one
[@qqwa]: https://github.com/qqwa
[@diggsey]: https://github.com/Diggsey
[@crajcan]: https://github.com/crajcan
[@demurgos]: https://github.com/demurgos
[@link2xt]: https://github.com/link2xt
[@guylapid]: https://github.com/guylapid
[@natproach]: https://github.com/NatPRoach
[@feikesteenbergen]: https://github.com/feikesteenbergen
[@etcaton]: https://github.com/ETCaton
[@toshokan]: https://github.com/toshokan
[@nomick]: https://github.com/nomick
[@marshoepial]: https://github.com/marshoepial
[@link2ext]: https://github.com/link2ext
[@madadam]: https://github.com/madadam
[@AtkinsChang]: https://github.com/AtkinsChang
[@djmarcin]: https://github.com/djmarcin
[@ghassmo]: https://github.com/ghassmo
[@eagletmt]: https://github.com/eagletmt
[@montanalow]: https://github.com/montanalow
[@nitnelave]: https://github.com/nitnelave
[@Drevoed]: https://github.com/Drevoed
[@yuyawk]: https://github.com/yuyawk
[@yerke]: https://github.com/yerke
[@russweas]: https://github.com/russweas
[@zbigniewzolnierowicz]: https://github.com/zbigniewzolnierowicz
[@dimfeld]: https://github.com/dimfeld
[@akiradeveloper]: https://github.com/akiradeveloper
[@chesedo]: https://github.com/chesedo
[@LLBlumire]: https://github.com/LLBlumire
[@liushuyu]: https://github.com/liushuyu
[@paolobarbolini]: https://github.com/paolobarbolini
[@DoumanAsh]: https://github.com/DoumanAsh
[@D1plo1d]: https://github.com/D1plo1d
[@tkintscher]: https://github.com/tkintscher
[@SonicZentropy]: https://github.com/SonicZentropy
[@parazyd]: https://github.com/parazyd
[@kunjee17]: https://github.com/kunjee17
[@05storm26]: https://github.com/05storm26
[@dbeckwith]: https://github.com/dbeckwith
[@k-jun]: https://github.com/k-jun
[@tranzystorek-io]: https://github.com/tranzystorek-io
[@taladar]: https://github.com/taladar
[@genusistimelord]: https://github.com/genusistimelord
[@p9s]: https://github.com/p9s
[@ArGGu]: https://github.com/ArGGu
[@sedrik]: https://github.com/sedrik
[@nappa85]: https://github.com/nappa85
[@ifn3]: https://github.com/ifn3
[@LovecraftianHorror]: https://github.com/LovecraftianHorror
[@stoically]: https://github.com/stoically
[@VersBinarii]: https://github.com/VersBinarii
[@cemoktra]: https://github.com/cemoktra
[@jdrouet]: https://github.com/jdrouet
[@vbmade2000]: https://github.com/vbmade2000
[@abreis]: https://github.com/abreis
[@0xdeafbeef]: https://github.com/0xdeafbeef
[@Dylan-DPC]: https://github.com/Dylan-DPC
[@carols10cents]: https://github.com/carols10cents
[@david-mcgillicuddy-moixa]: https://github.com/david-mcgillicuddy-moixa
[@ipetkov]: https://github.com/ipetkov
[@pedromfedricci]: https://github.com/pedromfedricci
[@tm-drtina]: https://github.com/tm-drtina
[@espindola]: https://github.com/espindola
[@mgrachev]: https://github.com/mgrachev
[@tyrelr]: https://github.com/tyrelr
[@SebastienGllmt]: https://github.com/SebastienGllmt
[@e00E]: https://github.com/e00E
[@sebpuetz]: https://github.com/sebpuetz
[@pruthvikar]: https://github.com/pruthvikar
[@tobymurray]: https://github.com/tobymurray
[@djc]: https://github.com/djc
[@mfreeborn]: https://github.com/mfreeborn
[@scottwey]: https://github.com/scottwey
[@e-rhodes]: https://github.com/e-rhodes
[@OskarPersson]: https://github.com/OskarPersson
[@walf443]: https://github.com/walf443
[@lovasoa]: https://github.com/lovasoa
[@mdtusz]: https://github.com/mdtusz
[@kianmeng]: https://github.com/kianmeng
[@EthanYuan]: https://github.com/EthanYuan
[@Nukesor]: https://github.com/Nukesor
[@smonv]: https://github.com/smonv
[@Erik1000]: https://github.com/Erik1000
[@raviqqe]: https://github.com/raviqqe
[@johnbcodes]: https://github.com/johnbcodes
[@sbeckeriv]: https://github.com/sbeckeriv
[@RomainStorai]: https://github.com/RomainStorai
[@jayy-lmao]: https://github.com/jayy-lmao
[@Thomasdezeeuw]: https://github.com/Thomasdezeeuw
[@kenkoooo]: https://github.com/kenkoooo
[@TheoOiry]: https://github.com/TheoOiry
[@JoeyMckenzie]: https://github.com/JoeyMckenzie
[@ivan]: https://github.com/ivan
[@crepererum]: https://github.com/crepererum
[@UramnOIL]: https://github.com/UramnOIL
[@liningpan]: https://github.com/liningpan
[@zzhengzhuo]: https://github.com/zzhengzhuo
[@crepererum]: https://github.com/crepererum
[@szymek156]: https://github.com/szymek156
[@NSMustache]: https://github.com/NSMustache
[@RustyYato]: https://github.com/RustyYato
[@alexander-jackson]: https://github.com/alexander-jackson
[@zlidner]: https://github.com/zlidner
[@zlindner]: https://github.com/zlindner
[@marcustut]: https://github.com/marcustut
[@rakshith-ravi]: https://github.com/rakshith-ravi
[@bradfier]: https://github.com/bradfier
[@fuzzbuck]: https://github.com/fuzzbuck
[@cycraig]: https://github.com/cycraig
[@fasterthanlime]: https://github.com/fasterthanlime
[@he4d]: https://github.com/he4d
[@DXist]: https://github.com/DXist
[@Wopple]: https://github.com/Wopple
[@TravisWhitehead]: https://github.com/TravisWhitehead
