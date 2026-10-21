use crate::error::{VppError, VppResult};
use crate::lexer::{Lexer, TokenKind};

/// Basic v++ formatter: normalizes spacing and indents braced blocks.
pub fn format(source: &str) -> VppResult<String> {
    let tokens = Lexer::new(source).tokenize()?;
    let mut out = String::new();
    let mut indent = 0usize;
    let mut prev_was_newline = true;
    let mut paren_depth = 0usize;

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let kind = &token.kind;

        if matches!(kind, TokenKind::Eof) {
            break;
        }

        if matches!(kind, TokenKind::Newline) {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            prev_was_newline = true;
            i += 1;
            continue;
        }

        if prev_was_newline && !matches!(kind, TokenKind::RBrace) {
            out.push_str(&"    ".repeat(indent));
            prev_was_newline = false;
        }

        match kind {
            TokenKind::LBrace => {
                out.push('{');
                indent += 1;
                if tokens.get(i + 1).is_some_and(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::RBrace)) {
                    out.push('\n');
                    prev_was_newline = true;
                }
            }
            TokenKind::RBrace => {
                indent = indent.saturating_sub(1);
                if !prev_was_newline {
                    out.push('\n');
                    out.push_str(&"    ".repeat(indent));
                }
                out.push('}');
                if tokens.get(i + 1).is_some_and(|t| !matches!(t.kind, TokenKind::Eof | TokenKind::RBrace | TokenKind::Newline)) {
                    out.push('\n');
                    prev_was_newline = true;
                }
            }
            TokenKind::Comma => {
                out.push(',');
                if paren_depth == 0 {
                    out.push(' ');
                }
            }
            TokenKind::Colon => out.push(':'),
            TokenKind::Arrow => out.push_str(" -> "),
            TokenKind::DotDot => out.push_str(".."),
            TokenKind::LParen => {
                paren_depth += 1;
                out.push('(');
            }
            TokenKind::RParen => {
                paren_depth = paren_depth.saturating_sub(1);
                out.push(')');
            }
            TokenKind::LBracket => out.push('['),
            TokenKind::RBracket => out.push(']'),
            TokenKind::Eq => out.push_str(" = "),
            TokenKind::EqEq => out.push_str(" == "),
            TokenKind::BangEq => out.push_str(" != "),
            TokenKind::Lt => out.push_str(" < "),
            TokenKind::LtEq => out.push_str(" <= "),
            TokenKind::Gt => out.push_str(" > "),
            TokenKind::GtEq => out.push_str(" >= "),
            TokenKind::AndAnd => out.push_str(" && "),
            TokenKind::OrOr => out.push_str(" || "),
            TokenKind::Plus => out.push_str(" + "),
            TokenKind::Minus => out.push_str(" - "),
            TokenKind::Star => out.push_str(" * "),
            TokenKind::Slash => out.push_str(" / "),
            TokenKind::Percent => out.push_str(" % "),
            TokenKind::Bang => out.push('!'),
            TokenKind::StringLit(s) => {
                out.push('"');
                out.push_str(s);
                out.push('"');
            }
            other => {
                out.push_str(&other.to_string());
                let next = tokens.get(i + 1).map(|t| &t.kind);
                if needs_space_after(other, next) {
                    out.push(' ');
                }
            }
        }

        i += 1;
    }

    if !out.ends_with('\n') {
        out.push('\n');
    }

    Ok(out)
}

fn needs_space_after(kind: &TokenKind, next: Option<&TokenKind>) -> bool {
    if matches!(
        next,
        Some(
            TokenKind::DotDot
                | TokenKind::Comma
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Colon
                | TokenKind::Newline
                | TokenKind::Eof
        )
    ) {
        return false;
    }

    matches!(
        kind,
        TokenKind::Let
            | TokenKind::Mut
            | TokenKind::Fn
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::While
            | TokenKind::For
            | TokenKind::In
            | TokenKind::Return
            | TokenKind::True
            | TokenKind::False
            | TokenKind::IntType
            | TokenKind::FloatType
            | TokenKind::BoolType
            | TokenKind::StringType
            | TokenKind::Ident(_)
    )
}

pub fn format_file(path: &std::path::Path) -> VppResult<()> {
    let source = std::fs::read_to_string(path).map_err(|source| VppError::Io { source })?;
    let formatted = format(&source)?;
    std::fs::write(path, formatted).map_err(|source| VppError::Io { source })
}

/// Minify v++ for LLM corpora: drop line breaks and indentation, keep semantics.
pub fn shrink(source: &str) -> VppResult<String> {
    let tokens = Lexer::new(source).tokenize()?;
    let mut out = String::new();
    let mut paren_depth = 0usize;
    let mut prev: Option<&TokenKind> = None;

    for token in &tokens {
        let kind = &token.kind;
        if matches!(kind, TokenKind::Eof | TokenKind::Newline) {
            continue;
        }

        if let Some(p) = prev {
            if needs_space_between(p, kind) {
                out.push(' ');
            }
        }

        match kind {
            TokenKind::LParen => {
                paren_depth += 1;
                out.push('(');
            }
            TokenKind::RParen => {
                paren_depth = paren_depth.saturating_sub(1);
                out.push(')');
            }
            TokenKind::LBrace => out.push('{'),
            TokenKind::RBrace => out.push('}'),
            TokenKind::LBracket => out.push('['),
            TokenKind::RBracket => out.push(']'),
            TokenKind::Comma => out.push(','),
            TokenKind::Colon => out.push(':'),
            TokenKind::Arrow => out.push_str("->"),
            TokenKind::DotDot => out.push_str(".."),
            TokenKind::Eq => out.push('='),
            TokenKind::EqEq => out.push_str("=="),
            TokenKind::BangEq => out.push_str("!="),
            TokenKind::Lt => out.push('<'),
            TokenKind::LtEq => out.push_str("<="),
            TokenKind::Gt => out.push('>'),
            TokenKind::GtEq => out.push_str(">="),
            TokenKind::AndAnd => out.push_str("&&"),
            TokenKind::OrOr => out.push_str("||"),
            TokenKind::Plus => out.push('+'),
            TokenKind::Minus => out.push('-'),
            TokenKind::Star => out.push('*'),
            TokenKind::Slash => out.push('/'),
            TokenKind::Percent => out.push('%'),
            TokenKind::Bang => out.push('!'),
            TokenKind::StringLit(s) => {
                out.push('"');
                out.push_str(s);
                out.push('"');
            }
            other => out.push_str(&other.to_string()),
        }

        prev = Some(kind);
    }

    Ok(out)
}

fn needs_space_between(prev: &TokenKind, next: &TokenKind) -> bool {
    if matches!(
        next,
        TokenKind::Comma
            | TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::RBrace
            | TokenKind::Colon
            | TokenKind::DotDot
    ) {
        return false;
    }
    if matches!(
        prev,
        TokenKind::LParen
            | TokenKind::LBrace
            | TokenKind::LBracket
            | TokenKind::Bang
            | TokenKind::Minus
    ) {
        return false;
    }
    needs_space_after(prev, Some(next))
}

/// Rough tokenizer count for LLM budget planning (one token ≈ one lexer symbol).
pub fn estimate_tokens(source: &str) -> VppResult<usize> {
    let tokens = Lexer::new(source).tokenize()?;
    Ok(tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Eof | TokenKind::Newline))
        .count())
}

pub fn shrink_file(path: &std::path::Path) -> VppResult<(String, String)> {
    let source = std::fs::read_to_string(path).map_err(|source| VppError::Io { source })?;
    let shrunk = shrink(&source)?;
    std::fs::write(path, &shrunk).map_err(|source| VppError::Io { source })?;
    Ok((source, shrunk))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_mut_and_range() {
        let out = format("let mut total = 0\nfor n in 1..6 { print(n) }").unwrap();
        assert!(out.contains("let mut total"), "got: {out}");
        assert!(out.contains("1..6"), "got: {out}");
        assert!(!out.contains("muttotal"));
        assert!(!out.contains("1 ..6"));
    }

    #[test]
    fn formats_block_indentation() {
        let out = format("if x > 0 { print(x) }").unwrap();
        assert!(out.contains("if x"));
        assert!(out.contains("{\n"));
    }

    #[test]
    fn shrink_drops_whitespace() {
        let src = "fn add(a: int, b: int) -> int {\n    return a + b\n}\n";
        let out = shrink(src).unwrap();
        assert!(!out.contains('\n'));
        assert!(out.contains("fn add"));
        assert!(out.len() < src.len());
    }

    #[test]
    fn estimate_tokens_counts_symbols() {
        let n = estimate_tokens("let x = 1").unwrap();
        assert!(n >= 4);
    }
}
