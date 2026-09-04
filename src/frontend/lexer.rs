use crate::frontend::{
    errors::LexerError,
    tokens::{self, TokenKind},
};
use regex::Regex;

pub type RegexHandler = fn(&mut Lexer, &Regex) -> Result<(), LexerError>;

pub struct RegexPattern {
    pub regex: Regex,
    pub kind: Option<TokenKind>,
    pub handler: Option<RegexHandler>,
}

impl RegexPattern {
    pub fn token(pattern: &str, kind: TokenKind) -> Self {
        Self {
            regex: Regex::new(pattern).unwrap(),
            kind: Some(kind),
            handler: None,
        }
    }

    pub fn handler(pattern: &str, handler: RegexHandler) -> Self {
        Self {
            regex: Regex::new(pattern).unwrap(),
            kind: None,
            handler: Some(handler),
        }
    }
}

pub struct Lexer {
    pub patterns: Vec<RegexPattern>,
    pub tokens: Vec<tokens::Token>,
    pub source: String,
    pub index: usize,
    pub pos: usize,
    pub line: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        create_lexer(source)
    }

    pub fn tokenize(&mut self) -> Result<Vec<tokens::Token>, LexerError> {
        while !self.at_eof() {
            let mut matched = false;

            for i in 0..self.patterns.len() {
                let found = {
                    let pattern = &self.patterns[i];
                    pattern
                        .regex
                        .find(self.remainder())
                        .map(|m| (m.start(), m.as_str().to_string()))
                };

                if let Some((0, text)) = found {
                    matched = true;

                    let kind = self.patterns[i].kind;
                    let handler = self.patterns[i].handler;
                    let regex = self.patterns[i].regex.clone();

                    if let Some(kind) = kind {
                        self.push(tokens::Token::new(kind, text.clone()));
                        self.advance_bytes(text.len());
                    } else if let Some(handler) = handler {
                        handler(self, &regex)?;
                    }

                    break;
                }
            }

            if !matched {
                Err(LexerError::UnexpectedCharacter {
                    line: self.line,
                    pos: self.pos,
                    character: self.remainder().chars().next().unwrap(),
                })?;
            }
        }

        self.push(tokens::Token::new(TokenKind::EOF, "EOF".to_string()));

        Ok(std::mem::take(&mut self.tokens))
    }

    fn advance_bytes(&mut self, n: usize) {
        let slice = &self.source[self.index..self.index + n];

        for ch in slice.chars() {
            if ch == '\n' {
                self.line += 1;
                self.pos = 0;
            } else {
                self.pos += 1;
            }
        }

        self.index += n;
    }

    fn remainder(&self) -> &str {
        &self.source[self.index..]
    }

    fn push(&mut self, token: tokens::Token) {
        self.tokens.push(token);
    }

    fn at_eof(&self) -> bool {
        self.index >= self.source.len()
    }
}

fn text_helper<'a>(source: &'a str, regex: &Regex) -> &'a str {
    regex.find(source).unwrap().as_str()
}

fn skip_handler(lex: &mut Lexer, regex: &Regex) -> Result<(), LexerError> {
    let text = text_helper(lex.remainder(), regex);

    lex.advance_bytes(text.len());

    Ok(())
}

fn string_handler(lex: &mut Lexer, regex: &Regex) -> Result<(), LexerError> {
    let text = regex.find(lex.remainder()).unwrap().as_str().to_string();

    lex.push(tokens::Token::new(
        TokenKind::STRING,
        text[1..text.len() - 1].to_string(),
    ));
    lex.advance_bytes(text.len());
    Ok(())
}

fn rune_handler(lex: &mut Lexer, regex: &Regex) -> Result<(), LexerError> {
    let text = regex.find(lex.remainder()).unwrap().as_str().to_string();

    lex.push(tokens::Token::new(TokenKind::RUNE, text.to_string()));
    lex.advance_bytes(text.len());
    Ok(())
}

fn number_handler(lex: &mut Lexer, regex: &Regex) -> Result<(), LexerError> {
    let text = regex.find(lex.remainder()).unwrap().as_str().to_string();
    let len = text.len();

    lex.push(tokens::Token::new(TokenKind::NUMBER, text));
    lex.advance_bytes(len);
    Ok(())
}

fn symbol_handler(lex: &mut Lexer, regex: &Regex) -> Result<(), LexerError> {
    let text = regex.find(lex.remainder()).unwrap().as_str().to_string();

    let kind = match tokens::KEYWORDS.get(text.as_str()) {
        Some(kind) => *kind,
        None => TokenKind::IDENT,
    };

    lex.push(tokens::Token::new(kind, text.clone()));
    lex.advance_bytes(text.len());
    Ok(())
}

fn comment_handler(lex: &mut Lexer, regex: &Regex) -> Result<(), LexerError> {
    let text = text_helper(lex.remainder(), regex);

    // Single-line comments shouldn't contain newlines,
    // but this doesn't hurt if the syntax changes.
    lex.advance_bytes(text.len());
    Ok(())
}

fn ml_comment_handler(lex: &mut Lexer, regex: &Regex) -> Result<(), LexerError> {
    let text = text_helper(lex.remainder(), regex);

    lex.advance_bytes(text.len());
    Ok(())
}

fn create_lexer(source: String) -> Lexer {
    Lexer {
        tokens: vec![],
        source,
        index: 0,
        pos: 0,
        line: 1,
        patterns: vec![
            // Whitespace
            RegexPattern::handler(r"\s+", skip_handler),
            // Comments
            RegexPattern::handler(r"(?s)##.*?##", ml_comment_handler),
            RegexPattern::handler(r"#[^#\n]*#", comment_handler),
            // Literals
            RegexPattern::handler(r#""(?:\\.|[^"\\])*""#, string_handler),
            RegexPattern::handler(r"<(?:\\.|[^\\>])>", rune_handler),
            RegexPattern::handler(r"[0-9]+(?:\.[0-9]+)?", number_handler),
            // Identifiers / Keywords
            RegexPattern::handler(r"[A-Za-z_][A-Za-z0-9_]*", symbol_handler),
            // Multi-character operators
            RegexPattern::token(r"===", TokenKind::TEQUAL),
            RegexPattern::token(r"==", TokenKind::EQUALS),
            RegexPattern::token(r"~=", TokenKind::NEQUAL),
            RegexPattern::token(r"<<<", TokenKind::LS),
            RegexPattern::token(r">>>", TokenKind::RS),
            RegexPattern::token(r">>", TokenKind::ACCESS),
            RegexPattern::token(r"<=", TokenKind::LE),
            RegexPattern::token(r">=", TokenKind::GE),
            RegexPattern::token(r"\+=", TokenKind::PE),
            RegexPattern::token(r"\*\*=", TokenKind::POWE),
            RegexPattern::token(r"-=", TokenKind::ME),
            RegexPattern::token(r"\*=", TokenKind::TE),
            RegexPattern::token(r"/=", TokenKind::SE),
            RegexPattern::token(r"%=", TokenKind::MODE),
            RegexPattern::token(r"\*\*", TokenKind::POW),
            RegexPattern::token(r"\.\.", TokenKind::UNTIL),
            RegexPattern::token(r"->", TokenKind::CONVERT),
            RegexPattern::token(r"\|\|", TokenKind::OR),
            RegexPattern::token(r"&&", TokenKind::AND),
            RegexPattern::token(r">--", TokenKind::OPP),
            RegexPattern::token(r">\+", TokenKind::INC),
            RegexPattern::token(r">-", TokenKind::DEC),
            RegexPattern::token(r">@", TokenKind::RZA),
            RegexPattern::token(r">!", TokenKind::FIB),
            // Unicode trig operators
            RegexPattern::token(r"σ", TokenKind::SIN),
            RegexPattern::token(r"γ", TokenKind::COS),
            RegexPattern::token(r"τ", TokenKind::TAN),
            // Single-character operators
            RegexPattern::token(r"=", TokenKind::ASSIGN),
            RegexPattern::token(r"~", TokenKind::NOT),
            RegexPattern::token(r"<", TokenKind::LT),
            RegexPattern::token(r">", TokenKind::GT),
            RegexPattern::token(r"\+", TokenKind::PLUS),
            RegexPattern::token(r"-", TokenKind::MINUS),
            RegexPattern::token(r"\*", TokenKind::TIMES),
            RegexPattern::token(r"/", TokenKind::SLASH),
            RegexPattern::token(r"%", TokenKind::MOD),
            RegexPattern::token(r"\^", TokenKind::XOR),
            RegexPattern::token(r"@", TokenKind::SQRT),
            RegexPattern::token(r"!", TokenKind::FACT),
            RegexPattern::token(r"\|", TokenKind::POLE),
            RegexPattern::token(r"\?", TokenKind::QUESTION),
            RegexPattern::token(r":", TokenKind::COLON),
            RegexPattern::token(r";", TokenKind::SC),
            RegexPattern::token(r",", TokenKind::COMMA),
            RegexPattern::token(r"#", TokenKind::POUND),
            // Delimiters
            RegexPattern::token(r"\(", TokenKind::SPAREN),
            RegexPattern::token(r"\)", TokenKind::CPAREN),
            RegexPattern::token(r"\[", TokenKind::SBRACK),
            RegexPattern::token(r"\]", TokenKind::CBRACK),
            RegexPattern::token(r"\{", TokenKind::SCURLY),
            RegexPattern::token(r"\}", TokenKind::CCURLY),
        ],
    }
}
