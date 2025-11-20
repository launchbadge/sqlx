create table "user"
(
    user_id       uuid primary key default gen_random_uuid(),
    username      text unique not null,
    password_hash text        not null
);
