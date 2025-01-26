# Axum App with Multi-tenant Database

This example project involves three crates, each owning a different schema in one database,
with their own set of migrations.

* The main crate, an Axum app. 
  * Owns the `public` schema (tables are referenced unqualified).
* `accounts`: a subcrate simulating a reusable account-management crate.
  * Owns schema `accounts`.
* `payments`: a subcrate simulating a wrapper for a payments API.
  * Owns schema `payments`.
