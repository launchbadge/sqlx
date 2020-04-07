# cargo-sqlx

Sqlx migrator runs all `*.sql` files under `migrations` folder and remembers which ones has been run. 

Database url is supplied through either env variable or `.env` file containing `DATABASE_URL="postgres://postgres:postgres@localhost/realworld"`.

##### Commands
- `add <name>` - add new migration to your migrations folder named `<timestamp>_<name>.sql` 
- `run` - Runs all migrations in your migrations folder


##### Limitations
- No down migrations! If you need down migrations, there are other more feature complete migrators to use.
- Only support postgres. Could be convinced to add other databases if there is need and easy to use database connection libs.
