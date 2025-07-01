create table purchase
(
    purchase_id uuid primary key     default gen_random_uuid(),
    account_id  uuid        not null references accounts.account (account_id),
    payment_id  uuid        not null references payments.payment (payment_id),
    amount      numeric     not null,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz
);

select trigger_updated_at('purchase');
