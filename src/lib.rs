use std::vec::Vec;
pub mod lex_stream;
use lex_stream::LexStream;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsToken {
    Whitespace(String),
    LineTerminator(String),
    LineComment(String),
    MultilineComment(String),
    Word(String),
    StringLiteral(String),
    NumberLiteral(String),
    RegexpLiteral(String, String),
    Punctuation(String),
    Unknown(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FsmState {
    Initial,
    AfterExpr,
    ExpectExpr,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct LexerVars {
    is_new_line: bool,
    last_token_disallows_newline: bool,
    last_token_nonexpr_paren: bool,
    last_token_for: bool,
}

pub struct JsTokenIterator<TIterator: Iterator<Item=char>> {
    char_iter: LexStream<TIterator>,
    state_stack: Vec<(char, FsmState, bool)>,
    state: FsmState,
    in_for: bool,
    lexer_vars: LexerVars,
}

pub fn tokenize_chars<TIterator: Iterator<Item=char>>(src: TIterator) -> JsTokenIterator<TIterator> {
    JsTokenIterator {
        char_iter: LexStream::new(src),
        state_stack: Vec::new(),
        state: FsmState::Initial,
        in_for: false,
        lexer_vars: LexerVars {
            is_new_line: true,
            last_token_disallows_newline: false,
            last_token_nonexpr_paren: false,
            last_token_for: false,
        },
    }
}

pub fn tokenize_str<'a>(src: &'a str) -> JsTokenIterator<std::str::Chars<'a>> {
    tokenize_chars(src.chars())
}

pub fn tokenize(src: &str) -> Vec<JsToken> {
    let tokenizer = tokenize_str(src);
    let result = tokenizer.collect();
    
    result
}

impl<TIterator: Iterator<Item=char>> JsTokenIterator<TIterator> {

    fn consume_number(&mut self) -> JsToken {
        let mut r = String::new();
        
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum State {
            Initial,
            InitialZero,
            Binary,
            Octal,
            Decimal,
            Hex,
        }
        
        let mut state = State::Initial;
        
        while let Some(c) = self.char_iter.lookahead(0) {
            //println!("consume_number: c = {:?}, state = {:?}", c, state);
            match (c, state) {
                ('0', State::Initial) => {
                    r.push(c);
                    self.char_iter.read();
                    state = State::InitialZero;
                },
                ('1'...'9', State::Initial) => {
                    r.push(c);
                    self.char_iter.read();
                    state = State::Decimal;
                },
                (_, State::Initial) => {
                    break;
                },
                ('b', State::InitialZero) |
                ('B', State::InitialZero) => {
                    r.push(c);
                    self.char_iter.read();
                    state = State::Binary;
                },
                ('o', State::InitialZero) |
                ('O', State::InitialZero) => {
                    r.push(c);
                    self.char_iter.read();
                    state = State::Octal;
                },
                ('x', State::InitialZero) |
                ('X', State::InitialZero) => {
                    r.push(c);
                    self.char_iter.read();
                    state = State::Hex;
                },
                ('0'...'1', State::Binary) |
                ('0'...'7', State::Octal) |
                ('0'...'9', State::InitialZero) |
                ('0'...'9', State::Decimal) |
                ('0'...'9', State::Hex)|
                ('a'...'f', State::Hex)|
                ('A'...'F', State::Hex) => {
                    r.push(c);
                    self.char_iter.read();
                },
                (_, _) => {
                    break;
                }
            }
        }
        
        
        if state == State::Decimal {
            if Some('.') == self.char_iter.lookahead(0) {
                r.push('.');
                self.char_iter.skip(1);
                while let Some(c) = self.char_iter.lookahead(0) {
                    match c {
                        '0'...'9' => {
                            r.push(c);
                            self.char_iter.skip(1);
                        },
                        _ => {
                            break;
                        },
                    }
                }
            }
            
            match self.char_iter.lookahead(0) {
                Some(c @ 'e') | Some(c @ 'E') => {
                    r.push(c);
                    self.char_iter.skip(1);
                    
                    match self.char_iter.lookahead(0) {
                        Some('-') => {
                            r.push('-');
                            self.char_iter.skip(1);
                        },
                        _ => { }
                    }
                    
                    while let Some(c) = self.char_iter.lookahead(0) {
                        match c {
                            '0'...'9' => {
                                r.push(c);
                                self.char_iter.skip(1);
                            },
                            _ => {
                                break;
                            }
                        }
                    }
                },
                _ => { }
            }
        }
        
        JsToken::NumberLiteral(r)
    }

    fn consume_whitespace(&mut self) -> JsToken {
        let mut result = String::new();
        while let Some(c) = self.char_iter.lookahead(0) {
            match c {
                ' '|'\t'|'\u{000b}'|'\u{000c}'|'\u{00a0}' => {
                    result.push(c);
                    self.char_iter.read();
                },
                _ => break,
            }
        }
        JsToken::Whitespace(result)
    }

    fn consume_string_literal(&mut self) -> JsToken {
        let mut r = String::new();
        
        #[derive(Copy, Clone)]
        enum QuoteKind {
            Single,
            Double,
        }
        
        let quote = match self.char_iter.read().unwrap() {
            '\'' => {
                r.push('\'');
                QuoteKind::Single
            },
            '\"' => {
                r.push('\"');
                QuoteKind::Double
            },
            _ => unreachable!(),
        };
        
        #[derive(Copy, Clone)]
        enum State {
            Initial,
            Backslash,
        }
        
        let mut state = State::Initial;
        
        while let Some(c) = self.char_iter.lookahead(0) {
            match (c, state, quote) {
                ('\'', State::Initial, QuoteKind::Single) |
                ('\"', State::Initial, QuoteKind::Double) => {
                    self.char_iter.read();
                    r.push(c);
                    break;
                },
                ('x', State::Backslash, _) => {
                    self.char_iter.read();
                    r.push(c);
                    for _ in 0..2 {
                        if let Some(c) = self.char_iter.read() {
                            r.push(c);
                        }
                    }
                    state = State::Initial;
                },
                ('u', State::Backslash, _) => {
                    self.char_iter.read();
                    r.push(c);
                    if Some('{') == self.char_iter.lookahead(0) {
                        r.push(self.char_iter.read().unwrap());
                        while let Some(c) = self.char_iter.read() {
                            r.push(c);
                            if c == '}' {
                                break;
                            }
                        }
                    } else {
                        for _ in 0..4 {
                            if let Some(c) = self.char_iter.read() {
                                r.push(c);
                            }
                        }
                    }
                    state = State::Initial;
                },
                ('\\', State::Initial, _) => {
                    self.char_iter.read();
                    r.push(c);
                    state = State::Backslash;
                },
                (_, _, _) => {
                    self.char_iter.read();
                    r.push(c);
                    state = State::Initial;
                },
            }
        }
        JsToken::StringLiteral(r)
    }

    fn consume_line_terminator(&mut self) -> JsToken {
        let mut result = String::new();
        while let Some(c) = self.char_iter.lookahead(0) {
            match c {
                '\r'|'\n' => {
                    result.push(c);
                    self.char_iter.read();
                },
                _ => break,
            }
        }
        JsToken::LineTerminator(result)
    }

    fn consume_word(&mut self) -> JsToken {
        let mut result = String::new();
        while let Some(c) = self.char_iter.lookahead(0) {
            if c == '_' || c == '$' || c.is_alphanumeric() {
                result.push(c);
                self.char_iter.read();
            } else {
                break;
            }
        }
        JsToken::Word(result)
    }

    fn consume_regexp(&mut self) -> JsToken {
        let mut result = String::new();
        let mut flags = String::new();
        result.push(self.char_iter.read().unwrap());
        let mut found_end = false;
        while let Some(c) = self.char_iter.lookahead(0) {
            match (c, self.char_iter.lookahead(1)) {
                ('\\', Some(c2)) => {
                    self.char_iter.skip(2);
                    result.push('\\');
                    result.push(c2);
                },
                ('/', _) => {
                    found_end = true;
                    self.char_iter.skip(1);
                    break;
                },
                _ => {
                    result.push(c);
                    self.char_iter.skip(1);
                },
            }
        }
        if found_end {
            while let Some(c) = self.char_iter.lookahead(0) {
                match c {
                    'a'...'z' => {
                        flags.push(c);
                        self.char_iter.skip(1);
                    },
                    _ => {
                        break;
                    },
                }
            }
        }
        JsToken::RegexpLiteral(result, flags)
    }

    fn consume_line_comment(&mut self) -> JsToken {
        self.char_iter.read();
        self.char_iter.read();
        let mut result = String::new();
        result.push(self.char_iter.read().unwrap());
        while let Some(c) = self.char_iter.lookahead(0) {
            if c == '\n' || c == '\r' {
                break;
            } else {
                result.push(c);
                self.char_iter.read();
            }
        }
        JsToken::LineComment(result)
    }

    fn consume_multiline_comment(&mut self) -> JsToken {
        self.char_iter.read();
        self.char_iter.read();
        let mut result = String::new();
        result.push(self.char_iter.read().unwrap());
        
        while let Some(c) = self.char_iter.lookahead(0) {
            match c {
                '*' => {
                    match self.char_iter.lookahead(1) {
                        Some('/') => {
                            self.char_iter.read();
                            self.char_iter.read();
                            break;
                        },
                        _ => {
                            result.push(c);
                            self.char_iter.read();
                        }
                    }
                }
                _ => {
                    result.push(c);
                    self.char_iter.read();
                }
            }
        }
        JsToken::MultilineComment(result)
    }
}

impl<TIterator: Iterator<Item=char>> Iterator for JsTokenIterator<TIterator> {
    type Item = JsToken;
    fn next(&mut self) -> Option<JsToken> {
        match self.char_iter.lookahead(0) {
            None => None,
            Some(c) => {
        
                //println!(
                //    "tokenize: c = {:?}, state = {:?}, in_for = {:?}, last_token_nonexpr_paren = {:?}",
                //    c, state, in_for,
                //    lexer_vars.last_token_nonexpr_paren);
                let token;
                match c {
                    '0'...'9' => {
                        token = self.consume_number();
                        self.state = FsmState::AfterExpr;
                    },
                    ' '|'\t'|'\u{000b}'|'\u{000c}'|'\u{00a0}' => token = self.consume_whitespace(),
                    '\r'|'\n' => token = self.consume_line_terminator(),
                    '\''|'"' => token = self.consume_string_literal(),
                    _ if c == '_' || c == '$' || c.is_alphabetic() => {
                        let word = self.consume_word();
                        if let JsToken::Word(ref w) = word {
                            if w == "return" || w == "yield" {
                                self.state = FsmState::ExpectExpr;
                            } else {
                                self.state = FsmState::AfterExpr;
                            }
                        } else {
                            self.state = FsmState::AfterExpr;
                        }
                        
                        token = word;
                    },
                    '(' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        let after_state = match (self.state, self.lexer_vars.last_token_nonexpr_paren) {
                            (_, true) => FsmState::Initial,
                            (FsmState::Initial, _) => FsmState::AfterExpr,
                            (FsmState::AfterExpr, _) => FsmState::AfterExpr,
                            (FsmState::ExpectExpr, _) => FsmState::AfterExpr,
                        };
                        self.state_stack.push((')', after_state, self.in_for));
                        self.in_for = if self.lexer_vars.last_token_for { true } else { false };
                        self.state = FsmState::ExpectExpr;
                    },
                    ')' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        let (_, st1, sc) = self.state_stack.pop().unwrap_or(('(', FsmState::Initial, false));
                        self.state = st1;
                        self.in_for = sc;
                    },
                    '{' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        let after_state = match self.state {
                            FsmState::Initial => FsmState::Initial,
                            FsmState::AfterExpr => FsmState::Initial,
                            FsmState::ExpectExpr => FsmState::AfterExpr,
                        };
                        self.state_stack.push(('}', after_state, self.in_for));
                        self.in_for = false;
                        self.state = FsmState::Initial;
                    },
                    '}' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        let (_, st1, sc) = self.state_stack.pop().unwrap_or(('{', FsmState::Initial, false));
                        self.state = st1;
                        self.in_for = sc;
                    },
                    '[' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        self.state_stack.push((']', FsmState::AfterExpr, self.in_for));
                        self.in_for = false;
                        self.state = FsmState::ExpectExpr;
                    },
                    ']' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        let (_, st1, sc) = self.state_stack.pop().unwrap_or(('[', FsmState::Initial, false));
                        self.state = st1;
                        self.in_for = sc;
                    },
                    
                    '!' => {
                        match (self.char_iter.lookahead(1), self.char_iter.lookahead(2)) {
                            (Some('='), Some('=')) => {
                                self.char_iter.skip(3);
                                token = JsToken::Punctuation("!==".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (Some('='), _) => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("!=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (_, _) => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("!".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '=' => {
                        match (self.char_iter.lookahead(1), self.char_iter.lookahead(2)) {
                            (Some('='), Some('=')) => {
                                self.char_iter.skip(3);
                                token = JsToken::Punctuation("===".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (Some('='), _) => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("==".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (_, _) => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '&' => {
                        match self.char_iter.lookahead(1) {
                            Some('&') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("&&".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            Some('=') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("&=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            _ => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("&".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '*' => {
                        match self.char_iter.lookahead(1) {
                            Some('=') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("*=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            _ => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("*".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '+' => {
                        match self.char_iter.lookahead(1) {
                            Some('=') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("+=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            Some('+') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("++".to_owned());
                                self.state = match (self.state, self.lexer_vars.is_new_line) {
                                    (FsmState::AfterExpr, false) => FsmState::AfterExpr, // postfix ++
                                    _ => FsmState::ExpectExpr, // prefix ++
                                }
                            },
                            _ => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("+".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '-' => {
                        match self.char_iter.lookahead(1) {
                            Some('=') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("-=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            Some('-') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("--".to_owned());
                                self.state = match (self.state, self.lexer_vars.is_new_line) {
                                    (FsmState::AfterExpr, false) => FsmState::AfterExpr, // postfix --
                                    _ => FsmState::ExpectExpr, // prefix --
                                }
                            },
                            _ => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("-".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '<' => {
                        match (self.char_iter.lookahead(1), self.char_iter.lookahead(2)) {
                            (Some('<'), Some('=')) => {
                                self.char_iter.skip(3);
                                token = JsToken::Punctuation("<<=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (Some('<'), _) => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("<<".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (Some('='), _) => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("<=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (_, _) => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("<".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '>' => {
                        match (self.char_iter.lookahead(1), self.char_iter.lookahead(2)) {
                            (Some('>'), Some('=')) => {
                                self.char_iter.skip(3);
                                token = JsToken::Punctuation(">>=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (Some('>'), _) => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation(">>".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (Some('='), _) => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation(">=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (_, _) => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation(">".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '|' => {
                        match self.char_iter.lookahead(1) {
                            Some('=') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("|=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            Some('|') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("||".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            _ => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("|".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '%' => {
                        match self.char_iter.lookahead(1) {
                            Some('=') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("%=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            _ => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("%".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    '^' => {
                        match self.char_iter.lookahead(1) {
                            Some('=') => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("^=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            _ => {
                                self.char_iter.skip(1);
                                token = JsToken::Punctuation("^".to_owned());
                                self.state = FsmState::ExpectExpr;
                            }
                        }
                    },
                    ','|':'|'~'|'?' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        self.state = FsmState::ExpectExpr;
                    },
                    '/' => {
                        match (self.char_iter.lookahead(1), self.state) {
                            (Some('/'), _) => token = self.consume_line_comment(),
                            (Some('*'), _) => token = self.consume_multiline_comment(),
                            (Some('='), _) => {
                                self.char_iter.skip(2);
                                token = JsToken::Punctuation("/=".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (_, FsmState::AfterExpr) => {
                                self.char_iter.read();
                                token = JsToken::Punctuation("/".to_owned());
                                self.state = FsmState::ExpectExpr;
                            },
                            (_, _) => {
                                self.char_iter.read();
                                token = self.consume_regexp();
                                self.state = FsmState::AfterExpr;
                            }
                        }
                    },
                    '.' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        self.state = FsmState::Initial;
                    },
                    ';' => {
                        self.char_iter.read();
                        token = JsToken::Punctuation(format!("{}", c));
                        if self.in_for {
                            self.state = FsmState::ExpectExpr;
                        } else {
                            self.state = FsmState::Initial;
                        }
                    },
                    _ => {
                        self.char_iter.read();
                        token = JsToken::Unknown(format!("{}", c));
                        self.state = FsmState::Initial;
                    },
                }
                
                //println!(" -> {:?}", token);
                
                match token {
                    JsToken::LineTerminator(_) => {
                        self.lexer_vars.is_new_line = true;
                    },
                    JsToken::MultilineComment(ref x) if x.contains("\n") => {
                        self.lexer_vars.is_new_line = true;
                    },
                    JsToken::Whitespace(_) |
                    JsToken::LineComment(_) => {
                        // nothing
                    },
                    _ => {
                        self.lexer_vars.is_new_line = false;
                    }
                }
                
                match token {
                    JsToken::LineTerminator(_) => {
                        if self.lexer_vars.last_token_disallows_newline {
                            self.state = FsmState::Initial;
                        }
                    },
                    JsToken::MultilineComment(ref x) if x.contains("\n") => {
                        if self.lexer_vars.last_token_disallows_newline {
                            self.state = FsmState::Initial;
                        }
                    },
                    JsToken::Word(ref x) if x == "return" || x == "continue" || x == "break" || x == "throw" || x == "yield" => {
                        self.lexer_vars.last_token_disallows_newline = true;
                        self.state = FsmState::ExpectExpr;
                    },
                    JsToken::Whitespace(_) |
                    JsToken::LineComment(_) => {
                        // nothing
                    },
                    _ => {
                        self.lexer_vars.last_token_disallows_newline = false;
                    }
                }
                
                match token {
                    JsToken::Word(ref x) if x == "if" || x == "for" || x == "while" => {
                        self.lexer_vars.last_token_nonexpr_paren = true;
                    },
                    JsToken::Whitespace(_) |
                    JsToken::LineComment(_) |
                    JsToken::LineTerminator(_) |
                    JsToken::MultilineComment(_) => {
                        // nothing
                    },
                    _ => {
                        self.lexer_vars.last_token_nonexpr_paren = false;
                    }
                }
                
                match token {
                    JsToken::Word(ref x) if x == "for" => {
                        self.lexer_vars.last_token_for = true;
                    },
                    JsToken::Whitespace(_) |
                    JsToken::LineComment(_) |
                    JsToken::LineTerminator(_) |
                    JsToken::MultilineComment(_) => {
                        // nothing
                    },
                    _ => {
                        self.lexer_vars.last_token_for = false;
                    }
                }
                    
                Some(token)
            }
        }
    }
}
