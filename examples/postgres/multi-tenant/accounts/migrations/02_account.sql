create table accounts.account
(
    account_id    uuid primary key     default gen_random_uuid(),
    email         text unique not null,
    password_hash text        not null,
    created_at    timestamptz not null default now(),
    updated_at    timestamptz
);

select accounts.trigger_updated_at('accounts.account');
