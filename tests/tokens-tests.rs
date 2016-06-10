extern crate js_lex_rs;

use js_lex_rs::*;

#[test]
pub fn whitespace() {
    let tokens = tokenize(" \t");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::Whitespace(" \t".to_string()), tokens[0]);
}

#[test]
pub fn line_terminator() {
    let tokens = tokenize("\r\n");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::LineTerminator("\r\n".to_string()), tokens[0]);
}

#[test]
pub fn line_comment() {
    let tokens = tokenize("//qwe");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::LineComment("qwe".to_string()), tokens[0]);
}

#[test]
pub fn line_comment_newline() {
    let tokens = tokenize("//qwe\n");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::LineComment("qwe".to_string()), tokens[0]);
    assert_eq!(JsToken::LineTerminator("\n".to_string()), tokens[1]);
}

#[test]
pub fn multiline_comment() {
    let tokens = tokenize("/* qwe* */");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::MultilineComment(" qwe* ".to_string()), tokens[0]);
}

#[test]
pub fn word() {
    let tokens = tokenize("abc null false");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::Word("abc".to_string()), tokens[0]);
    assert_eq!(JsToken::Whitespace(" ".to_string()), tokens[1]);
    assert_eq!(JsToken::Word("null".to_string()), tokens[2]);
    assert_eq!(JsToken::Whitespace(" ".to_string()), tokens[3]);
    assert_eq!(JsToken::Word("false".to_string()), tokens[4]);
}

#[test]
pub fn num_binary() {
    let tokens = tokenize("0b001");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::NumberLiteral("0b001".to_string()), tokens[0]);
}

#[test]
pub fn num_octal() {
    let tokens = tokenize("0O01237");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::NumberLiteral("0O01237".to_string()), tokens[0]);
}

#[test]
pub fn num_decimal() {
    let tokens = tokenize("1234");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::NumberLiteral("1234".to_string()), tokens[0]);
}

#[test]
pub fn num_decimal_zero() {
    let tokens = tokenize("01234");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::NumberLiteral("01234".to_string()), tokens[0]);
}

#[test]
pub fn num_hex() {
    let tokens = tokenize("0x0ABCf");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::NumberLiteral("0x0ABCf".to_string()), tokens[0]);
}

#[test]
pub fn string_single_plain() {
    let tokens = tokenize("'qwe' ");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::StringLiteral("'qwe'".to_string()), tokens[0]);
}

#[test]
pub fn string_double_plain() {
    let tokens = tokenize("\"qwe\" ");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::StringLiteral("\"qwe\"".to_string()), tokens[0]);
}

#[test]
pub fn test_single_double() {
    let tokens = tokenize("'\"' ");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::StringLiteral("'\"'".to_string()), tokens[0]);
}

#[test]
pub fn test_double_single() {
    let tokens = tokenize("\"'\" ");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::StringLiteral("\"'\"".to_string()), tokens[0]);
}

#[test]
pub fn test_escape() {
    let tokens = tokenize("'\\'' ");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::StringLiteral("'\\''".to_string()), tokens[0]);
}

fn tokenize_no_whitespace(s: &str) -> Vec<JsToken> {
    let tokens: Vec<_> = tokenize(s)
        .into_iter()
        .filter(|x|
            match *x {
                JsToken::Whitespace(_) |
                JsToken::LineTerminator(_) |
                JsToken::MultilineComment(_) |
                JsToken::LineComment(_) => false,
                _ => true,
            }
        )
        .collect();

    tokens
}

#[test]
pub fn test_exclamation() {
    let tokens = tokenize_no_whitespace("! != !==");
    assert_eq!(3, tokens.len());
    assert_eq!(JsToken::Punctuation("!".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("!=".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("!==".to_string()), tokens[2]);
}

#[test]
pub fn test_eq() {
    let tokens = tokenize_no_whitespace("= == ===");
    assert_eq!(3, tokens.len());
    assert_eq!(JsToken::Punctuation("=".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("==".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("===".to_string()), tokens[2]);
}

#[test]
pub fn test_and() {
    let tokens = tokenize_no_whitespace("& && &=");
    assert_eq!(3, tokens.len());
    assert_eq!(JsToken::Punctuation("&".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("&&".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("&=".to_string()), tokens[2]);
}

#[test]
pub fn test_plus() {
    let tokens = tokenize_no_whitespace("+ ++ +=");
    assert_eq!(3, tokens.len());
    assert_eq!(JsToken::Punctuation("+".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("++".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("+=".to_string()), tokens[2]);
}

#[test]
pub fn test_plus_plus_prefix() {
    let tokens = tokenize_no_whitespace("++{}/2");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::Punctuation("++".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("{".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("}".to_string()), tokens[2]);
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[3]);
    assert_eq!(JsToken::NumberLiteral("2".to_string()), tokens[4]);
}

#[test]
pub fn test_plus_plus_suffix() {
    let tokens = tokenize_no_whitespace("1++/2");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::NumberLiteral("1".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("++".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[2]);
    assert_eq!(JsToken::NumberLiteral("2".to_string()), tokens[3]);
}

#[test]
pub fn test_minus() {
    let tokens = tokenize_no_whitespace("- -- -=");
    assert_eq!(3, tokens.len());
    assert_eq!(JsToken::Punctuation("-".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("--".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("-=".to_string()), tokens[2]);
}

#[test]
pub fn test_minus_minus_prefix() {
    let tokens = tokenize_no_whitespace("--{}/2");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::Punctuation("--".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("{".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("}".to_string()), tokens[2]);
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[3]);
    assert_eq!(JsToken::NumberLiteral("2".to_string()), tokens[4]);
}

#[test]
pub fn test_minus_minus_suffix() {
    let tokens = tokenize_no_whitespace("1--/2");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::NumberLiteral("1".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("--".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[2]);
    assert_eq!(JsToken::NumberLiteral("2".to_string()), tokens[3]);
}

#[test]
pub fn test_multiply() {
    let tokens = tokenize_no_whitespace("* *=");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::Punctuation("*".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("*=".to_string()), tokens[1]);
}

#[test]
pub fn test_div() {
    let tokens = tokenize_no_whitespace("1/");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::NumberLiteral("1".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[1]);
}

#[test]
pub fn test_div_eq() {
    let tokens = tokenize_no_whitespace("/=");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::Punctuation("/=".to_string()), tokens[0]);
}

#[test]
pub fn test_less_than() {
    let tokens = tokenize_no_whitespace("< <= <<= <<");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::Punctuation("<".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("<=".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation("<<=".to_string()), tokens[2]);
    assert_eq!(JsToken::Punctuation("<<".to_string()), tokens[3]);
}

#[test]
pub fn test_greater_than() {
    let tokens = tokenize_no_whitespace("> >= >>= >>");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::Punctuation(">".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation(">=".to_string()), tokens[1]);
    assert_eq!(JsToken::Punctuation(">>=".to_string()), tokens[2]);
    assert_eq!(JsToken::Punctuation(">>".to_string()), tokens[3]);
}

#[test]
pub fn test_power() {
    let tokens = tokenize_no_whitespace("^ ^=");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::Punctuation("^".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("^=".to_string()), tokens[1]);
}

#[test]
pub fn test_modulo() {
    let tokens = tokenize_no_whitespace("% %=");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::Punctuation("%".to_string()), tokens[0]);
    assert_eq!(JsToken::Punctuation("%=".to_string()), tokens[1]);
}

#[test]
pub fn test_regexp() {
    let tokens = tokenize_no_whitespace("/qwe/gi");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("qwe".to_string(), "gi".to_string()), tokens[0]);
}

#[test]
pub fn test_regexp_div_disambig_1() {
    let tokens = tokenize_no_whitespace("/qwe/");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("qwe".to_string(), "".to_string()), tokens[0]);
}

#[test]
pub fn test_regexp_div_disambig_2() {
    let tokens = tokenize_no_whitespace("{}++/qwe/");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("qwe".to_string(), "".to_string()), tokens[3]);
}

#[test]
pub fn test_regexp_div_disambig_3() {
    let tokens = tokenize_no_whitespace("++{}/qwe");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[3]);
}

#[test]
pub fn test_regexp_div_disambig_4() {
    let tokens = tokenize_no_whitespace("1+()/qwe");
    assert_eq!(6, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[4]);
}

#[test]
pub fn test_regexp_div_disambig_5() {
    let tokens = tokenize_no_whitespace("()/qwe");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[2]);
}

#[test]
pub fn test_float() {
    let tokens = tokenize_no_whitespace("1 1.0 1e1 1.0e1 1.0e-1");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::NumberLiteral("1".to_string()), tokens[0]);
    assert_eq!(JsToken::NumberLiteral("1.0".to_string()), tokens[1]);
    assert_eq!(JsToken::NumberLiteral("1e1".to_string()), tokens[2]);
    assert_eq!(JsToken::NumberLiteral("1.0e1".to_string()), tokens[3]);
    assert_eq!(JsToken::NumberLiteral("1.0e-1".to_string()), tokens[4]);
}

#[test]
pub fn test_string_escape() {
    let tokens = tokenize_no_whitespace("'\\n\\\\\\''");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::StringLiteral("'\\n\\\\\\''".to_string()), tokens[0]);
}

#[test]
pub fn test_string_escape_hex() {
    let tokens = tokenize_no_whitespace("'\\xAA\\xBB'");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::StringLiteral("'\\xAA\\xBB'".to_string()), tokens[0]);
}

#[test]
pub fn test_string_escape_utf16() {
    let tokens = tokenize_no_whitespace("'\\u12AB'");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::StringLiteral("'\\u12AB'".to_string()), tokens[0]);
}

#[test]
pub fn test_string_escape_unicode() {
    let tokens = tokenize_no_whitespace("'\\u{1234ABCD}'");
    assert_eq!(1, tokens.len());
    assert_eq!(JsToken::StringLiteral("'\\u{1234ABCD}'".to_string()), tokens[0]);
}

#[test]
pub fn test_string_increment_prefix() {
    let tokens = tokenize_no_whitespace("1\n++{}/q");
    assert_eq!(6, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[4]);
}

#[test]
pub fn test_string_increment_suffix() {
    let tokens = tokenize_no_whitespace("1++{}/q/");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("q".to_string(), "".to_string()), tokens[4]);
}

#[test]
pub fn test_string_decrement_prefix() {
    let tokens = tokenize_no_whitespace("1\n--{}/q");
    assert_eq!(6, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[4]);
}

#[test]
pub fn test_string_decrement_suffix() {
    let tokens = tokenize_no_whitespace("1--{}/q/");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("q".to_string(), "".to_string()), tokens[4]);
}

#[test]
pub fn test_string_paren_return_regexp() {
    let tokens = tokenize_no_whitespace("return /qwe/");
    assert_eq!(2, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("qwe".to_string(), "".to_string()), tokens[1]);
}

#[test]
pub fn test_string_paren_return_expr_regexp() {
    let tokens = tokenize_no_whitespace("return {}/qwe/");
    assert_eq!(6, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[3]);
}

#[test]
pub fn test_string_paren_return_nl_expr_regexp() {
    let tokens = tokenize_no_whitespace("return\n{}/qwe/");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("qwe".to_string(), "".to_string()), tokens[3]);
}

#[test]
pub fn test_string_paren_return_id_div() {
    let tokens = tokenize_no_whitespace("returnqq /qwe/");
    assert_eq!(4, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[1]);
}

#[test]
pub fn test_string_paren_regexp_if() {
    let tokens = tokenize_no_whitespace("if (1)/qwe/");
    assert_eq!(5, tokens.len());
    assert_eq!(JsToken::RegexpLiteral("qwe".to_string(), "".to_string()), tokens[4]);
}

#[test]
pub fn test_string_paren_regexp_funcall() {
    let tokens = tokenize_no_whitespace("iff (1)/qwe");
    assert_eq!(6, tokens.len());
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[4]);
}

#[test]
pub fn test_for_regexp() {
    let tokens = tokenize_no_whitespace("for({}/1;{}/1;{}/1)");
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[4]);
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[9]);
    assert_eq!(JsToken::Punctuation("/".to_string()), tokens[14]);
}

const JQUERY_SRC: &'static str = include_str!("jquery-1.12.4.js");
const JQUERY_MIN_SRC: &'static str = include_str!("jquery-1.12.4.min.js");

#[test]
pub fn lex_jquery() {
    let tokens = tokenize(JQUERY_SRC);
    for token in &tokens {
        if let &JsToken::Unknown(_) = token {
            assert!(false, "Found unknown token: {:?}", token);
        }
    }
}


#[test]
pub fn lex_jquery_min() {
    let tokens = tokenize(JQUERY_MIN_SRC);
    for token in &tokens {
        if let &JsToken::Unknown(_) = token {
            assert!(false, "Found unknown token: {:?}", token);
        }
    }
}
