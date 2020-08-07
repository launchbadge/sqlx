//! Conversions between Rust and **PostgreSQL** types.

mod integer;
// mod decimal;
// mod real;

// Reference documentation:
// https://www.postgresql.org/docs/current/datatype.html

// To lookup the encoding for a type, use the following query:
//  SELECT typreceive, typsend, typinput, typoutput FROM pg_type WHERE typname = 'NAME';

// each refers to a function name in the postgres source code
//  typreceive is the binary decoder
//  typsend is the binary encoder
//  typinput is the text decoder
//  typoutput is the text encoder
