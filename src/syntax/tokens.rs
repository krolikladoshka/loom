use maplit::hashmap;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub enum TokenType {
    #[default]
    EOF,

    Identifier,
    Keyword,
    F32Literal,
    F64Literal,
    I8Literal,
    I16Literal,
    I32Literal,
    I64Literal,
    U8Literal,
    U16Literal,
    U32Literal,
    U64Literal,
    BoolLiteral,
    StringLiteral,
    MultilineStringLiteral,
    CharLiteral,
    ArrayLiteral,

    Vector2Literal,
    Vector3Literal,
    Vector4Literal,

    Matrix2Literal,
    Matrix3Literal,
    Matrix4Literal,

    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    V2,
    V3,
    V4,
    M2x2,
    M3x3,
    M4x4,

    Comma,
    Dot,
    Semicolon,
    Colon,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParenthesis,
    RightParenthesis,
    SingleQuote,
    DoubleQuote,
    StrokeQuote,

    Plus, Minus,
    Star, Slash,
    Percent,

    Less,
    Greater,
    Equals,
    LessEqual,
    GreaterEqual,
    Ampersand,

    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    BinaryOrEquals,
    BinaryAndEquals,
    BinaryXorEquals,
    BinaryInvertEquals,

    Arrow, // ->
    MatchClauseSeparator, // =>
    QualifierSeparator, // ::
    RangeExclusive, // ..
    RangeInclusive, // ..=
    SingleLineComment, // //
    MultilineCommentStart, // /*
    MultilineCommentEnd, // */
    Increment, // ++
    Decrement, // --
    EqualsEquals,
    NotEquals,
    LogicalOr, // ||
    LogicalAnd, // &&
    LogicalNot, // !
    BinaryOr, // |
    BinaryAnd, // &
    BinaryXor, // ^,
    BinaryInvert, // ~
    BinaryShiftLeft, // <<
    BinaryShiftRight, // >>

    Pub,
    Use,
    Extern,
    Enum,
    Union,
    Struct,
    Impl,
    Branch,
    Go,

    EnumStruct, // enum struct
    UnionStruct, // union struct
    
    SelfToken,
    Fn,
    Let,
    Mut,
    Static,
    Const,
    Defer,
    Block,
    Loop,
    For,
    While,
    Break,
    Continue,
    If,
    Else,
    Return,
    Match,
    As,
    Raw,
    AsRaw, // as row <type> // reinterpret_cast
}

impl TokenType {
    pub const UNARY_OPERATORS: &'static [Self] = &[
        Self::Star,
        Self::Minus,
        Self::LogicalNot,
        Self::BinaryInvert,
        Self::Increment,
        Self::Decrement,
    ];
    
    pub const MULTIPLICATIVE_OPERATORS: &'static [Self] = &[
        Self::Star,
        Self::Slash,
        Self::Percent,
    ];
    
    pub const ADDITIVE_OPERATORS: &'static [Self] = &[
        Self::Plus,
        Self::Minus,
    ];
    
    pub const BINARY_SHIFT_OPERATORS: &'static [Self] = &[
        Self::BinaryShiftLeft,
        Self::BinaryShiftRight,
    ];
    
    pub const COMPARISON_OPERATORS: &'static [Self] = &[
        Self::Less,
        Self::LessEqual,
        Self::Greater,
        Self::GreaterEqual,
        Self::EqualsEquals,
        Self::NotEquals,
    ];
    
    pub const RANGE_OPERATORS: &'static [Self] = &[
        Self::RangeExclusive,
        Self::RangeInclusive,
    ];
    
    pub const ACCESS_OPERATORS: &'static [Self] = &[
        Self::Dot,
        Self::Arrow,
    ];
}

pub static SIMPLE_TOKENS: LazyLock<HashMap<char, TokenType>> = LazyLock::new(
    || {
        hashmap! {
            ',' => TokenType::Comma,
            ':' => TokenType::Colon,
            ';' => TokenType::Semicolon,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            '(' => TokenType::LeftParenthesis,
            ')' => TokenType::RightParenthesis,
            '!' => TokenType::LogicalNot,
        }
    }
);

pub static DOUBLE_TOKENS: LazyLock<HashMap<char, (HashMap<char, TokenType>, TokenType)>> =
    LazyLock::new(
    || {
        hashmap! {
            '-' => (
                hashmap! {
                    '-' => TokenType::Decrement,
                    '>' => TokenType::Arrow,
                    '=' => TokenType::MinusEquals
                },
                TokenType::Minus,
            ),
            '+' => (
                hashmap! {
                    '+' => TokenType::Increment,
                    '=' => TokenType::PlusEquals,
                },
                TokenType::Plus,
            ),
            '*' => (
                hashmap! {
                    '=' => TokenType::StarEquals,
                    '/' => TokenType::MultilineCommentEnd,
                },
                TokenType::Star,
            ),
            '/' => (
                hashmap! {
                    '=' => TokenType::SlashEquals,
                    '/' => TokenType::SingleLineComment,
                    '*' => TokenType::MultilineCommentStart,
                },
                TokenType::Slash,
            ),
            '|' => (
                hashmap! {
                    '|' => TokenType::LogicalOr,
                    '=' => TokenType::BinaryOrEquals,
                },
                TokenType::BinaryOr
            ),
            '&' => (
                hashmap! {
                    '&' => TokenType::LogicalAnd,
                    '=' => TokenType::BinaryAndEquals
                },
                TokenType::Ampersand,
            ),
            '.' => (
                hashmap! {
                    '.' => TokenType::RangeExclusive,
                    '=' => TokenType::RangeInclusive
                },
                TokenType::Dot
            ),
            '!' => (
                hashmap! {
                    '=' => TokenType::NotEquals,
                },
                TokenType::LogicalNot,
            ),
            '=' => (
                hashmap! {
                    '=' => TokenType::EqualsEquals,
                },
                TokenType::Equals
            ),
            '~' => (
                hashmap! {
                    '=' => TokenType::BinaryInvertEquals
                },
                TokenType::BinaryInvert,
            ),
            '<' => (
                hashmap! {
                    '=' => TokenType::LessEqual,
                    '<' => TokenType::BinaryShiftLeft,
                },
                TokenType::Less,
            ),
            '>' => (
                hashmap! {
                    '=' => TokenType::GreaterEqual,
                    '>' => TokenType::BinaryShiftRight,
                },
                TokenType::Greater,
            )
        }
    }
);


pub static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(
    || {
        hashmap! {
            "i8" => TokenType::I8,
            "i16" => TokenType::I16,
            "i32" => TokenType::I32,
            "i64" => TokenType::I64,
            "u8" => TokenType::U8,
            "u16" => TokenType::U16,
            "u32" => TokenType::U32,
            "u64" => TokenType::U64,
            "f32" => TokenType::F32,
            "f64" => TokenType::F64,
            "bool" => TokenType::Bool,
            "v2" => TokenType::V2,
            "v3" => TokenType::V3,
            "v4" => TokenType::V4,
            "m2x2" => TokenType::M2x2,
            "m3x3" => TokenType::M3x3,
            "m4x4" => TokenType::M4x4,
            "pub" => TokenType::Pub,
            "extern" => TokenType::Extern,
            "enum" => TokenType::Enum,
            "union" => TokenType::Union,
            "struct" => TokenType::Struct,
            "impl" => TokenType::Impl,
            "self" => TokenType::SelfToken,
            "fn" => TokenType::Fn,
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "static" => TokenType::Static,
            "const" => TokenType::Const,
            "defer" => TokenType::Defer,
            "loop" => TokenType::Loop,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "return" => TokenType::Return,
            "match" => TokenType::Match,
            "as" => TokenType::As,
            "raw" => TokenType::Raw,
            "go" => TokenType::Go,
            "branch" => TokenType::Go,
        }
    }
);