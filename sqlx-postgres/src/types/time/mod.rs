mod date;
mod datetime;
mod time;

#[rustfmt::skip]
const PG_EPOCH: ::time::Date = ::time::macros::date!(2000-1-1);
