create table accounts.session
(
    session_token text primary key, -- random alphanumeric string
    account_id    uuid        not null references accounts.account (account_id),
    created_at    timestamptz not null default now()
);
