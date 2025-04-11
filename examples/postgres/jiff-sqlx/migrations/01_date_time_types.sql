create table date_time_types (
    -- `jiff::civil::Date`
    date_column date not null,
    -- `jiff::civil::Time`
    time_column time not null,
    -- `jiff::civil::DateTime`
    datetime_column timestamp not null,
    -- `jiff::Timestamp`
    timestamp_column timestamptz not null,
    -- `jiff::Span`; note: only decode supported
    span_column interval not null
);