// One of several questionable design decisions in MySQL is the choice to conflate
// *how stored data is sorted* with the character encoding used over the wire.
//
// The documentation for `Protocol::HandshakeResponse41` implies that
// the lower 8 bits of the collation ID may be used to uniquely identify the character set:
// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_handshake_response.html
//
// However, this isn't at _all_ true in practice. Collation IDs are assigned without any apparent
// rhyme or reason, mostly just sequential with unexplained gaps. Masking the collation ID with 0xFF
// doesn't actually tell you anything meaningful, except obviously for collation IDs under 256
// which just gives you the same collation ID again.
//
// Hanlon's razor would suggest they just forgot that they told clients they could do this.
// Occam's razor suggests no one ever bothers to set the connection charset/collation this way,
// and they all just default to `latin1_swedish_ci` (8), `utf8mb4_general_ci` (45),
// or `utf8mb4_0900_ai_ci` (255).
//
// This would seem to mean that if we want to be *sure* of the character encoding of a given column,
// we have to reference the _full_ catalog of collations. Because new ones are added occasionally,
// we can't just assume a collation we don't recognize is UTF-8 as that's not always the case.
//
// This is especially true when we include MariaDB because they've started creating
// their *own* collations, and even character sets, separately from MySQL.
//
// Awesome, right?
//
// However, as long as `character_set_client` and `character_set_results` are set correctly,
// we can assume that any non-binary collation is a valid string, because the server will transcode.
// As it turns out, the collation specified in the `Protocol::ColumnDefinition`
// is *purely* informational. It has no bearing on what's sent over the wire except for `binary` (63),
// which is never transcoded.
//
// So at the end of the day, none of this matters anyway! To know if a column is a string or not,
// we merely need to check if it's not `binary` (63). If the protocol was just a *bit*
// better documented, it would have saved me literally six hours spent figuring this out.
//
// Thanks, MySQL.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Deserialize, serde::Serialize))]
pub struct Collation(pub u16);

impl Collation {
    /// Collation used for all non-string data.
    pub const BINARY: Self = Collation(63);

    /// Most broadly supported UTF-8 collation.
    pub const UTF8MB4_GENERAL_CI: Self = Collation(45);
}
