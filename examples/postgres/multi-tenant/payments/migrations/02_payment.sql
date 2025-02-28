-- `payments::PaymentStatus`
--
-- Historically at LaunchBadge we preferred not to define enums on the database side because it can be annoying
-- and error-prone to keep them in-sync with the application.
-- Instead, we let the application define the enum and just have the database store a compact representation of it.
-- This is mostly a matter of taste, however.
--
-- For the purposes of this example, we're using an in-database enum because this is a common use-case
-- for needing type overrides.
create type payments.payment_status as enum (
    'pending',
    'created',
    'success',
    'failed'
    );

create table payments.payment
(
    payment_id          uuid primary key                 default gen_random_uuid(),
    -- This cross-schema reference means migrations for the `accounts` crate should be run first.
    account_id          uuid                    not null references accounts.account (account_id),

    status              payments.payment_status not null,

    -- ISO 4217 currency code (https://en.wikipedia.org/wiki/ISO_4217#List_of_ISO_4217_currency_codes)
    --
    -- This *could* be an ENUM of currency codes, but constraining this to a set of known values in the database
    -- would be annoying to keep up to date as support for more currencies is added.
    --
    -- Consider also if support for cryptocurrencies is desired; those are not covered by ISO 4217.
    --
    -- Though ISO 4217 is a three-character code, `TEXT`, `VARCHAR` and `CHAR(N)`
    -- all use the same storage format in Postgres. Any constraint against the length of this field
    -- would purely be a sanity check.
    currency            text                    not null,
    -- There's an endless debate about what type should be used to represent currency amounts.
    --
    -- Postgres has the `MONEY` type, but the fractional precision depends on a C locale setting and the type is mostly
    -- optimized for storing USD, or other currencies with a minimum fraction of 1 cent.
    --
    -- NEVER use `FLOAT` or `DOUBLE`. IEEE-754 rounding point has round-off and precision errors that make it wholly
    -- unsuitable for representing real money amounts.
    --
    -- `NUMERIC`, being an arbitrary-precision decimal format, is a safe default choice that can support any currency,
    -- and so is what we've chosen here.
    amount              NUMERIC                 not null,

    -- Payments almost always take place through a third-party vendor (e.g. PayPal, Stripe, etc.),
    -- so imagine this is an identifier string for this payment in such a vendor's systems.
    --
    -- For privacy and security reasons, payment and personally-identifying information
    -- (e.g. credit card numbers, bank account numbers, billing addresses) should only be stored with the vendor
    -- unless there is a good reason otherwise.
    external_payment_id text,
    created_at          timestamptz             not null default now(),
    updated_at          timestamptz
);

select payments.trigger_updated_at('payments.payment');
