//! Source code sanitization for tree-sitter parsing.
//!
//! Some language constructs are not supported by tree-sitter grammars and require
//! preprocessing before parsing. This module handles those edge cases.

use std::borrow::Cow;

use super::language::SupportedLanguage;

/// Sanitize source code for tree-sitter parsing.
///
/// Applies language-specific transformations to work around tree-sitter grammar limitations:
/// - TypeScript: `export type * from` → `export * from` (TS 5.0 re-export syntax)
/// - TSX: Escapes `&` in JSX text nodes (XML requirement)
pub(crate) fn sanitize_for_tree_sitter(
    source_text: &str,
    language: SupportedLanguage,
) -> Cow<'_, str> {
    match language {
        SupportedLanguage::TypeScript => {
            // tree-sitter-typescript doesn't reliably parse TS 5.0 `export type * from ...` yet.
            if source_text.contains("export type * from") {
                Cow::Owned(source_text.replace("export type * from", "export * from"))
            } else {
                Cow::Borrowed(source_text)
            }
        }
        SupportedLanguage::Tsx => {
            let mut result = Cow::Borrowed(source_text);

            if result.contains("export type * from") {
                result = Cow::Owned(result.replace("export type * from", "export * from"));
            }

            // tree-sitter's TSX grammar treats JSX text as XML, so `&` must be escaped.
            // Many TSX codebases include raw `&`/`&&` in JSX text nodes; sanitize for parsing only.
            if result.contains('&') && result.contains('<') {
                let escaped = escape_ampersands_in_jsx_text(&result);
                if escaped != *result {
                    result = Cow::Owned(escaped);
                }
            }

            result
        }
        _ => Cow::Borrowed(source_text),
    }
}

fn escape_ampersands_in_jsx_text(source: &str) -> String {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum State {
        Normal,
        InTag,
        InText,
        InExpr,
    }

    fn is_ident_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
    }

    fn prev_non_ws_char(chars: &[char], mut i: usize) -> Option<char> {
        while i > 0 {
            i -= 1;
            let ch = chars[i];
            if !ch.is_whitespace() {
                return Some(ch);
            }
        }
        None
    }

    fn looks_like_entity(chars: &[char], i: usize) -> bool {
        if i >= chars.len() || chars[i] != '&' {
            return false;
        }
        let next = match chars.get(i + 1) {
            Some(c) => *c,
            None => return false,
        };

        if next == '#' {
            // &#123; or &#x1F600;
            let mut j = i + 2;
            if matches!(chars.get(j), Some('x') | Some('X')) {
                j += 1;
                let mut has_hex = false;
                while let Some(c) = chars.get(j) {
                    if c.is_ascii_hexdigit() {
                        has_hex = true;
                        j += 1;
                        continue;
                    }
                    break;
                }
                return has_hex && matches!(chars.get(j), Some(';'));
            }

            let mut has_digit = false;
            while let Some(c) = chars.get(j) {
                if c.is_ascii_digit() {
                    has_digit = true;
                    j += 1;
                    continue;
                }
                break;
            }
            return has_digit && matches!(chars.get(j), Some(';'));
        }

        if !next.is_ascii_alphabetic() {
            return false;
        }

        let mut j = i + 2;
        while let Some(c) = chars.get(j) {
            if c.is_ascii_alphanumeric() {
                j += 1;
                continue;
            }
            break;
        }

        matches!(chars.get(j), Some(';'))
    }

    fn is_jsx_tag_start_in_text(chars: &[char], i: usize) -> bool {
        if chars.get(i) != Some(&'<') {
            return false;
        }
        let next = match chars.get(i + 1) {
            Some(c) => *c,
            None => return false,
        };

        next.is_ascii_alphabetic() || next == '/' || next == '>'
    }

    fn is_probably_jsx_tag_start_in_normal(chars: &[char], i: usize) -> bool {
        if !is_jsx_tag_start_in_text(chars, i) {
            return false;
        }

        !matches!(prev_non_ws_char(chars, i), Some(prev) if is_ident_char(prev) || prev == '.' || prev == ')' || prev == ']')
    }

    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());

    let mut state = State::Normal;
    let mut jsx_depth: usize = 0;
    let mut jsx_entry_stack: Vec<usize> = Vec::new();
    let mut tag_quote: Option<char> = None;
    let mut tag_brace_depth: usize = 0;
    let mut expr_depth: usize = 0;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        match state {
            State::Normal => {
                if is_probably_jsx_tag_start_in_normal(&chars, i) {
                    state = State::InTag;
                    tag_quote = None;
                    tag_brace_depth = 0;
                }
                out.push(ch);
                i += 1;
            }
            State::InTag => {
                out.push(ch);

                if let Some(q) = tag_quote {
                    if ch == q {
                        tag_quote = None;
                    }
                    i += 1;
                    continue;
                }

                match ch {
                    '"' | '\'' => tag_quote = Some(ch),
                    '{' => tag_brace_depth += 1,
                    '}' => tag_brace_depth = tag_brace_depth.saturating_sub(1),
                    '>' if tag_brace_depth == 0 => {
                        // Determine if this tag changes depth.
                        let mut j = i;
                        while j > 0 && chars[j].is_whitespace() {
                            j -= 1;
                        }
                        let mut k = i;
                        while k > 0 {
                            k -= 1;
                            if chars[k] == '<' {
                                break;
                            }
                        }
                        let is_closing = chars.get(k + 1) == Some(&'/');
                        let self_closing = !is_closing && chars.get(j) == Some(&'/');

                        if self_closing {
                            // no-op
                        } else if is_closing {
                            jsx_depth = jsx_depth.saturating_sub(1);
                        } else {
                            jsx_depth += 1;
                        }

                        if matches!(jsx_entry_stack.last(), Some(&entry) if jsx_depth == entry) {
                            jsx_entry_stack.pop();
                            state = State::InExpr;
                        } else {
                            state = if jsx_depth == 0 {
                                State::Normal
                            } else {
                                State::InText
                            };
                        }
                    }
                    _ => {}
                }

                i += 1;
            }
            State::InText => match ch {
                '{' => {
                    state = State::InExpr;
                    expr_depth = 1;
                    out.push(ch);
                    i += 1;
                }
                '<' if is_jsx_tag_start_in_text(&chars, i) => {
                    state = State::InTag;
                    tag_quote = None;
                    tag_brace_depth = 0;
                    out.push(ch);
                    i += 1;
                }
                '&' => {
                    if looks_like_entity(&chars, i) {
                        out.push('&');
                    } else {
                        out.push_str("&amp;");
                    }
                    i += 1;
                }
                _ => {
                    out.push(ch);
                    i += 1;
                }
            },
            State::InExpr => {
                if ch == '<' && is_probably_jsx_tag_start_in_normal(&chars, i) {
                    jsx_entry_stack.push(jsx_depth);
                    state = State::InTag;
                    tag_quote = None;
                    tag_brace_depth = 0;
                    out.push(ch);
                    i += 1;
                    continue;
                }

                out.push(ch);
                match ch {
                    '{' => expr_depth += 1,
                    '}' => {
                        expr_depth = expr_depth.saturating_sub(1);
                        if expr_depth == 0 {
                            state = State::InText;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_typescript_export_type_star() {
        let input = r#"export type * from "../documents/types";"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::TypeScript);
        assert!(sanitized.contains("export * from"));
        assert!(!sanitized.contains("export type * from"));
    }

    #[test]
    fn test_sanitize_tsx_escapes_ampersand_in_jsx_text() {
        let input = r#"export const C = () => (<div>Effects & Animations</div>);"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::Tsx);
        assert!(sanitized.contains("Effects &amp; Animations"));
    }

    #[test]
    fn test_sanitize_tsx_escapes_ampersand_in_jsx_inside_expression() {
        let input = r#"
export const C = () => (
  <div>
    {cond ? (<p>a & b</p>) : (<p>c && d</p>)}
  </div>
);
"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::Tsx);
        assert!(sanitized.contains("<p>a &amp; b</p>"));
        assert!(sanitized.contains("<p>c &amp;&amp; d</p>"));
    }

    #[test]
    fn test_sanitize_tsx_does_not_escape_type_intersection() {
        let input = r#"
export const X = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
    showCloseButton?: boolean;
  }
>(() => (<div>a & b</div>));
"#;
        let sanitized = sanitize_for_tree_sitter(input, SupportedLanguage::Tsx);
        assert!(sanitized.contains("ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {"));
        assert!(sanitized.contains("<div>a &amp; b</div>"));
    }
}
