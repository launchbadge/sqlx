use sqlx::Sqlite;
use sqlx_test::test_type;
use std::fmt::Debug;

// Transparent types are rust-side wrappers over DB types
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(transparent)]
struct Transparent(i32);

// "Weak" enums map to an integer type indicated by #[repr]
#[derive(PartialEq, Copy, Clone, Debug, sqlx::Type)]
#[repr(i32)]
enum Weak {
    One = 0,
    Two = 2,
    Three = 4,
}

// "Strong" enums can map to TEXT or a custom enum
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
enum Color {
    Red,
    Green,
    Blue,
}

test_type!(transparent(
    Sqlite,
    Transparent,
    "0" == Transparent(0),
    "23523" == Transparent(23523)
));

test_type!(weak_enum(
    Sqlite,
    Weak,
    "0" == Weak::One,
    "2" == Weak::Two,
    "4" == Weak::Three
));

test_type!(strong_color_enum(Sqlite, Color, "'green'" == Color::Green));
