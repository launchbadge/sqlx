# Contributing to SQLx

Thank you for your interest in contributing to SQLx! 
There are many ways to contribute and we appreciate all of them.

This page contains information about reporting issues as well as
some tips and guidelines useful to experienced open source contributors.

## Getting Started

> todo: how to setup the project, run tests, etc.

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
