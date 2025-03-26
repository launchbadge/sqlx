-- no-transaction

CREATE TABLE test_table (x int);
-- split-migration
CREATE INDEX CONCURRENTLY test_table_x_idx ON test_table (x);
-- split-migration
INSERT INTO test_table (x) VALUES (1);
-- prove that you can have a comment that won't split -- split-migration DROP TABLE does_not_exist;
