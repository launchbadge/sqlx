# SQLx Quickstart

## Purpose
Many developers want to get started with the basics fast. This is a guide to show the common usecases for using real world SQLx, without overcomplications of over-abstraction.

### Requirements
- Rust 1.8 or higher
- Database (described below)

#### Database

This guide utilizes the chinook database, a fictional dataset that showcases the common complexities of a relational database in many forms. This particular quickstart is based on MySQL.

The database can be stood up automatically and locally via the following
`docker-compose up -d`

If a mysql database exists, the schema and data can be loaded manually via:

```mysql -u root -h 127.0.0.1 -p < https://raw.githubusercontent.com/lerocha/chinook-database/master/ChinookDatabase/DataSources/Chinook_MySql.sql```

[MySQL Schema And Data](https://github.com/lerocha/chinook-database/blob/master/ChinookDatabase/DataSources/Chinook_MySql.sql)

## General Notes

SQLx maps rust types to rust native data types. See [here](https://docs.rs/sqlx/latest/sqlx/mysql/types/index.html).