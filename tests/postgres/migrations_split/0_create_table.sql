-- no-transaction

CREATE TABLE test_table (x int);
/* sqlx: split */
CREATE INDEX CONCURRENTLY test_table_x_idx ON test_table (x);
/* sqlx: split */
INSERT INTO test_table (x) VALUES (1);
