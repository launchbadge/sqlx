create table session
(
    session_token text primary key, -- random alphanumeric string
    account_id    uuid        not null references account (account_id),
    created_at    timestamptz not null default now()
);
