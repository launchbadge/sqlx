CREATE TABLE IF NOT EXISTS widgets (
  id       BIGSERIAL PRIMARY KEY,
  serial   BIGINT NOT NULL,
  name     TEXT UNIQUE NOT NULL,
  description TEXT NOT NULL
);
INSERT INTO widgets ( serial, name, description )
VALUES
( 10138, 'spanner', 'blue 10 guage joint spanner'),
( 39822, 'flexarm', 'red flexible support arm'),
( 52839, 'bearing', 'steel bearing for articulating joints');
