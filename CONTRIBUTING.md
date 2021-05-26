# Contributing to SQLx

Thank you for your interest in contributing to SQLx! 
There are many ways to contribute and we appreciate all of them.

This page contains information about reporting issues as well as
some tips and guidelines useful to experienced open source contributors.

## Getting Started

SQLx uses a Python3 script called `x.py` to run `check`, `test`, etc. on 
all of the crates in the workspace in all of the supported 
feature configurations.

- Run `check` for all postgres variants (incl. examples)

    ```sh
    $ ./x.py check:postgres
    ```

- Run `check` for _only_ blocking postgres

    ```sh
    # -e is short for --exact
    $ ./x.py -e check:postgres:blocking
    ```

- Run `check` for all async variants

    ```sh
    $ ./x.py check:_:async
    ```
  
- Run `unit` tests

    ```sh
    $ ./x.py unit
    ```
  
- Use `-l` or `--list` to list all tasks.

## Conventions

We aim to be consistent in our naming of both lifetimes 
and type parameters.

### Lifetimes

| Lifetime | Description |
| --- | --- |
| `'x` | Single e**x**ecution; lifetime ends after the `fetch`, `execute`, etc |
| `'v` | Argument **v**alue |
| `'q` | SQL query string |
| `'e` | Executor |
| `'c` | Connection |
| `'p` | Pool |
| `'t` | Transaction |

### Type Parameters

| Type Parameter | Bound | Description |
| --- | --- | --- |
| `Rt` | `Runtime` | |
| `E` | `Executor` | |
| `X` | `Execute` | |
| `Db` | `Database` | |
| `C` | `Connection` | |
| `R` | `Row` | |
| `O` | `FromRow` | |
| `T` | `Encode`, `Decode`, and/or `Type` | |
