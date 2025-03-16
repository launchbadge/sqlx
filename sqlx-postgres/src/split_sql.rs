use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar_inline = r#"
// The top-level rule matches the entire SQL input
sql = { SOI ~ statement* ~ EOI }

// A statement consists of optional leading comments and whitespace, content, and is terminated by a semicolon or end of input
statement = { (WHITESPACE | COMMENT)* ~ statement_content ~ (semicolon | &EOI) }

// Statement content is a sequence of constructs, comments, whitespace, or non-construct characters
statement_content = { (construct | COMMENT | WHITESPACE | non_construct_char)+ }

// Constructs that may contain semicolons internally
construct = { DOLLAR_QUOTED_STRING | SINGLE_QUOTED_STRING | DOUBLE_QUOTED_IDENTIFIER }

// Non-construct characters are any characters except semicolons
non_construct_char = { !semicolon ~ ANY }

// Semicolon outside constructs acts as a statement terminator
semicolon = { ";" }

// Single-quoted string literals, handling escaped quotes
SINGLE_QUOTED_STRING = { "'" ~ SINGLE_QUOTED_CONTENT ~ ("'" | EOI) }
SINGLE_QUOTED_CONTENT = { ( "''" | !("'" | EOI) ~ ANY )* }

// Double-quoted identifiers, handling escaped quotes
DOUBLE_QUOTED_IDENTIFIER = { "\"" ~ DOUBLE_QUOTED_IDENTIFIER_CONTENT ~ ("\"" | EOI) }
DOUBLE_QUOTED_IDENTIFIER_CONTENT = { ( "\"\"" | !("\"" | EOI) ~ ANY )* }

// Dollar-quoted strings, handling custom tags
DOLLAR_QUOTED_STRING = { DOLLAR_QUOTE_START ~ DOLLAR_QUOTED_CONTENT ~ DOLLAR_QUOTE_END }
DOLLAR_QUOTE_START = { "$" ~ DOLLAR_QUOTE_TAG ~ "$" }
DOLLAR_QUOTE_TAG = { ASCII_ALPHANUMERIC* }
DOLLAR_QUOTED_CONTENT = { ( !DOLLAR_QUOTE_END ~ ANY )* }
DOLLAR_QUOTE_END = { "$" ~ DOLLAR_QUOTE_TAG ~ "$" }

// Comments (single-line and multi-line)
COMMENT = { SINGLE_LINE_COMMENT | MULTI_LINE_COMMENT }
SINGLE_LINE_COMMENT = { "--" ~ (!NEWLINE ~ ANY)* ~ NEWLINE? }

MULTI_LINE_COMMENT = { "/*" ~ MULTI_LINE_COMMENT_CONTENT* ~ ( "*/" | EOI ) }
MULTI_LINE_COMMENT_CONTENT = { MULTI_LINE_COMMENT | (!"/*" ~ !"*/" ~ ANY) }

// Whitespace rules
WHITESPACE = { " " | "\t" | NEWLINE }
NEWLINE = { "\r\n" | "\n" | "\r" }
"#]
struct PsqlSpliter;

/// Splits a PostgreSQL query string into it's individual statements.
///
/// This function parses and splits a SQL input string into separate statements, handling
/// PostgreSQL-specific syntax elements such as:
///
/// - **Dollar-quoted strings**: Supports custom dollar-quoted tags (e.g., `$$`, `$tag$`).
/// - **Single and double-quoted strings**: Handles escaped quotes inside strings.
/// - **Comments**: Supports single-line (`--`) and multi-line (`/* ... */`) comments, preserving them as part of the statement.
/// - **Whitespace**: Retains all leading and trailing whitespace and comments around each statement.
/// - **Semicolons**: Recognizes semicolons as statement terminators, while ignoring them inside strings or comments.
///
/// If parsing fails or only one statement is found, the input is returned in full.
///
/// ```no_run
/// use sql_split_pest::split_psql;
/// let sql = r#"
///     -- First query
///     INSERT INTO users (name) VALUES ('Alice; Bob');
///
///     -- Second query
///     SELECT * FROM posts;
///
///     /* Multi-line
///     comment */
///     CREATE FUNCTION test_function()
///     RETURNS VOID AS $$
///     BEGIN
///         -- Multiple statements inside the function
///         INSERT INTO table_a VALUES (1);
///         INSERT INTO table_b VALUES (2);
///     END;
///     $$ LANGUAGE plpgsql;
///
///     -- invalid sql
///     SELECT 'This is an unterminated string FROM users;
///     SELECT * FROM users WHERE name = AND email = 'john@example.com';
///     SELECT * FROM users JOIN other_table ON;
///
/// "#;
///
/// let statements = split_psql(sql);
/// dbg!(&statements);
/// assert_eq!(statements.len(), 4);
/// assert!(statements[0].contains("INSERT INTO users"));
/// assert!(statements[1].contains("SELECT * FROM posts"));
/// assert!(statements[2].contains("CREATE FUNCTION"));
/// assert!(statements[2].contains("plpgsql"));
/// assert!(statements[3].contains("other_table"));
/// ```
pub fn split_sql<S: AsRef<str>>(sql: S) -> Vec<String> {
    let sql_str = sql.as_ref();

    PsqlSpliter::parse(Rule::sql, sql_str).map_or_else(
        |_| vec![sql_str.to_string()],
        |mut parsed| match parsed.next() {
            // this should never happen
            None => vec![sql_str.to_string()],
            Some(sql) => {
                let mut statements = Vec::new();
                let mut statement = String::new();
                for pair in sql.into_inner() {
                    match pair.as_rule() {
                        Rule::WHITESPACE | Rule::COMMENT => statement.push_str(pair.as_str()),
                        Rule::statement | Rule::EOI => {
                            statement.push_str(pair.as_str());
                            // omit empty whitespace at the end of sql
                            if !statement.is_empty() && !statement.chars().all(char::is_whitespace)
                            {
                                statements.push(std::mem::take(&mut statement));
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                statements
            }
        },
    )
}
