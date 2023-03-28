
#[derive(Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub filename: String,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}:{}:{}", self.filename, self.line, self.column)
    }
}

impl std::fmt::Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct Span(pub Location, pub Location);

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} - {}", self.0, self.1)
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)
    }
}


impl Span {
    pub fn extend(&self, other: &Span) -> Span {
        Span(self.0.clone(), other.1.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    And,
    Assert,
    At,
    Bang,
    BangEquals,
    Colon,
    Comma,
    Def,
    Dot,
    DotDot,
    EOF,
    Else,
    Equals,
    EqualsEquals,
    False,
    FatArrow,
    FloatLiteral,
    For,
    Identifier,
    If,
    In,
    IntegerLiteralBin,
    IntegerLiteralOct,
    IntegerLiteralDec,
    IntegerLiteralHex,
    LeftBrace,
    LeftBracket,
    LeftParen,
    LessThan,
    LessEquals,
    Let,
    Minus,
    Not,
    GreaterThan,
    GreaterEquals,
    Nothing,
    Or,
    Pipe,
    Plus,
    Return,
    RightBrace,
    RightBracket,
    RightParen,
    SemiColon,
    Slash,
    Star,
    StringLiteral,
    True,
    While,
    Continue,
    Break,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
    pub newline_before: bool,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: String) -> Token {
        Token {
            kind,
            span,
            text,
            newline_before: false,
        }
    }

    pub fn from_str(text: String, span: Span) -> Token {
        Token {
            kind: match text.as_ref() {
                "and" => TokenKind::And,
                "assert" => TokenKind::Assert,
                "def" => TokenKind::Def,
                "else" => TokenKind::Else,
                "false" => TokenKind::False,
                "if" => TokenKind::If,
                "let" => TokenKind::Let,
                "not" => TokenKind::Not,
                "nothing" => TokenKind::Nothing,
                "or" => TokenKind::Or,
                "return" => TokenKind::Return,
                "true" => TokenKind::True,
                "while" => TokenKind::While,
                "continue" => TokenKind::Continue,
                "break" => TokenKind::Break,
                "for" => TokenKind::For,
                "in" => TokenKind::In,
                _ => TokenKind::Identifier,
            },
            span,
            text,
            newline_before: false,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.kind)?;
        if !self.text.is_empty() {
            write!(f, "({})", self.text)?;
        }
        Ok(())
    }
}
