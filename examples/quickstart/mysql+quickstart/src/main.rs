use sqlx::mysql::MySqlPool;
use sqlx::FromRow;

use sqlx::error::Error as sqlx_error;

/// Represents the Customer table in Chinook. Not all columns of the Customer table need to be present to unmarshall into this struct.
#[derive(Debug, FromRow)]
pub struct Customer {
    #[sqlx(rename = "CustomerId")]
    pub id: i32,
    #[sqlx(rename = "FirstName")]
    pub first_name: String,
    #[sqlx(rename = "LastName")]
    pub last_name: String,
}

/// This struct is to store a JOIN between Customer and Representative (Employee table)
#[derive(Debug, FromRow)]
pub struct CustomerRep {
    // The rename trait informs SQLx of the literal column name in the database
    #[sqlx(rename = "CustomerFirstName")]
    pub customer_first_name: String,
    #[sqlx(rename = "CustomerLastName")]
    pub customer_last_name: String,
    #[sqlx(rename = "RepFirstName")]
    pub rep_first_name: String,
    #[sqlx(rename = "RepLastName")]
    pub rep_last_name: String,
    #[sqlx(rename = "RepTitle")]
    pub rep_title: String,
}

/// This is a custom enum to hold various SQLx errors. Additional types of errors can be added here
#[derive(Debug)]
enum SQLError {
    NoRowsFoundError,
    OtherDBError,
}

impl From<sqlx_error> for SQLError {
    /// Transforms a SQLX library error into a custom error
    fn from(err: sqlx_error) -> SQLError {
        match err {
            sqlx_error::RowNotFound => SQLError::NoRowsFoundError,
            _ => SQLError::OtherDBError,
        }
    }
}

#[tokio::main]
pub async fn main() {
    let pool = create_db_pool().await;

    select_one(&pool).await;
    select_many(&pool).await;
    join_select(&pool).await;
    insert(&pool).await;
    delete(&pool).await;
    transaction(&pool).await;
    update(&pool).await;

    let no_records_err = error_handling_no_records(&pool).await;

    match no_records_err {
        Ok(n) => println!("Rows found: {:?}", n),
        Err(err) => println!("No rows found!"),
    }
}

/// This creates a database pool. Typically, the connection string would be an environment variable.
async fn create_db_pool() -> MySqlPool {
    let db_connection_string = "mysql://root:mysecretpassword@localhost/Chinook";
    let db_pool = MySqlPool::connect(&db_connection_string).await;

    return db_pool.unwrap();
}

/// Selects one record from the database and marshalls it into a rust struct.
async fn select_one(pool: &MySqlPool) {
    // Note: When defining the struct (in this case Customer), not all the fields need to be defined to use SELECT *
    let sql = "SELECT * FROM Customer WHERE CustomerId = ?";

    let customer_id: i32 = 1;

    let customer: Customer = sqlx::query_as(sql)
        .bind(customer_id)
        .fetch_one(pool)
        .await
        .unwrap();

    println!("{:?}", customer);
}

/// Selects many records from the database and unmarshalls it into a Vec rust struct.
async fn select_many(pool: &MySqlPool) {
    let sql = "SELECT * FROM Customer WHERE CustomerId > 50";

    let customers: Vec<Customer> = sqlx::query_as(sql).fetch_all(pool).await.unwrap();

    println!("{:?}", customers);
}

/// Selects many records and unmarshalls them into a rust struct.
async fn join_select(pool: &MySqlPool) {
    let sql = "SELECT c.FirstName as \"CustomerFirstName\", c.LastName as \"CustomerLastName\", e.FirstName as \"RepFirstName\" , e.LastName as \"RepLastName\" , e.Title as \"RepTitle\" FROM Customer c JOIN Employee e ON (e.EmployeeId = c.SupportRepId)";

    let customers: Vec<CustomerRep> = sqlx::query_as(sql).fetch_all(pool).await.unwrap();

    println!("{:?}", customers);
}

/// Inserts a record into the database.
async fn insert(pool: &MySqlPool) {
    let name = "Luis".to_string();
    let sql = "INSERT INTO Chinook.Customer (CustomerId,FirstName,LastName,Company,Address,City,State,Country,PostalCode,Phone,Fax,Email,SupportRepId) VALUES
    (100000,?,'Gonçalves','Embraer - Empresa Brasileira de Aeronáutica S.A.','Av. Brigadeiro Faria Lima, 2170','São José dos Campos','SP','Brazil','12227-000','+55 (12) 3923-5555','+55 (12) 3923-5566','luisg@embraer.com.br',3)";

    sqlx::query(sql).bind(name).execute(pool).await.unwrap();
}

/// Removes a record from the database.
async fn delete(pool: &MySqlPool) {
    let sql = "DELETE FROM Chinook.Customer WHERE CustomerId = 100000";

    sqlx::query(sql).execute(pool).await.unwrap();
}

// Performs a database transaction.
async fn transaction(pool: &MySqlPool) {
    let mut tx = pool.begin().await.unwrap();

    let sql = "SELECT * FROM Customer";

    sqlx::query(sql).execute(&mut tx).await.unwrap();

    // Transactions are rolled back if commit is not called when the function calls Drop(). See:
    // https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html
    tx.commit().await.unwrap();
}

/// Performs an update on the database.
async fn update(pool: &MySqlPool) {
    let sql = "UPDATE Customer SET FirstName = ? WHERE CustomerId = 2";

    sqlx::query(sql).bind("John").execute(pool).await.unwrap();
}

/// This is to show how to properly handle errors. In this case, we expect to find no records, which returns an error that is transformed
/// into a custom error enum. Normally, we recommend using fetch_all (even if expecting one record), as this will return an empty vector as opposed to an error.
async fn error_handling_no_records(pool: &MySqlPool) -> Result<Customer, SQLError> {
    let customer_id: i32 = 123123;
    let sql = "SELECT * FROM Customer WHERE CustomerId = ?";

    let query: Customer = sqlx::query_as(sql)
        .bind(customer_id)
        .fetch_one(pool)
        .await?;

    Ok(query)
}
