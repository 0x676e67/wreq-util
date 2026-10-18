//! Parsing and serializing of structured field values, as far as client hints need it.
//!
//! <https://www.rfc-editor.org/rfc/rfc9651>

use wreq::header::{HeaderMap, HeaderName};

use super::store::Accepted;

/// The most hints listed for one origin. Chrome knows fewer than this.
const MAX_HINTS: usize = 64;

/// Serializes `value` as a structured field string, which only holds printable ASCII.
pub(super) fn quote(value: &str) -> Option<String> {
    if !value.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
        return None;
    }

    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for c in value.chars() {
        if matches!(c, '"' | '\\') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted.push('"');
    Some(quoted)
}

/// Reads a structured field string from the start of `input`, returning it with what follows.
fn unquote(input: &str) -> Option<(String, &str)> {
    let mut chars = input.strip_prefix('"')?.char_indices();
    let mut value = String::new();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((value, &input[i + 2..])),
            // Only a quote or a backslash can be escaped.
            '\\' => match chars.next()?.1 {
                escaped @ ('"' | '\\') => value.push(escaped),
                _ => return None,
            },
            _ => value.push(c),
        }
    }
    None
}

/// Parses the brands and versions of a `Sec-CH-UA` value.
pub(super) fn parse_brands(value: &str) -> Option<Vec<(String, String)>> {
    let mut brands = Vec::new();
    let mut rest = value.trim_start_matches([' ', '\t']);
    while !rest.is_empty() {
        let (brand, after_brand) = unquote(rest)?;
        let version = after_brand
            .trim_start_matches([' ', '\t'])
            .strip_prefix(';')?
            .trim_start_matches([' ', '\t'])
            .strip_prefix("v=")?;
        let (version, after_version) = unquote(version)?;
        brands.push((brand, version));
        rest = after_version.trim_start_matches([' ', '\t']);
        if !rest.is_empty() {
            rest = rest.strip_prefix(',')?.trim_start_matches([' ', '\t']);
            // A list doesn't end with a comma.
            if rest.is_empty() {
                return None;
            }
        }
    }
    (!brands.is_empty()).then_some(brands)
}

/// Returns the tokens of a structured field list, without their parameters.
pub(super) fn tokens(value: &str) -> Option<Vec<&str>> {
    let mut tokens = Vec::new();
    let mut rest = value.trim_matches(' ');
    while !rest.is_empty() {
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && !"!#$%&'*+-.^_`|~:/".contains(c))
            .unwrap_or(rest.len());
        let (token, after_token) = rest.split_at(end);
        if !token.starts_with(|c: char| c.is_ascii_alphabetic() || c == '*') {
            return None;
        }
        tokens.push(token);

        rest = skip_parameters(after_token)?.trim_start_matches([' ', '\t']);
        if !rest.is_empty() {
            rest = rest.strip_prefix(',')?.trim_start_matches([' ', '\t']);
            // A list doesn't end with a comma.
            if rest.is_empty() {
                return None;
            }
        }
    }
    Some(tokens)
}

/// Parses the client hints listed in an `Accept-CH` or `Critical-CH` response header.
///
/// Returns `None` if the response doesn't have the header, or if it isn't a list of tokens, which
/// Chrome ignores as well.
pub(super) fn listed(headers: &HeaderMap, name: &HeaderName) -> Option<Accepted> {
    if !headers.contains_key(name) {
        return None;
    }

    let mut hints = Vec::new();
    for value in headers.get_all(name) {
        for token in tokens(value.to_str().ok()?)? {
            // A token is the name of the request header.
            let Ok(hint) = HeaderName::from_bytes(token.as_bytes()) else {
                continue;
            };
            if hints.len() < MAX_HINTS && !hints.contains(&hint) {
                hints.push(hint);
            }
        }
    }
    Some(hints.into())
}

/// Skips the parameters of a structured field item, returning what follows them.
fn skip_parameters(mut rest: &str) -> Option<&str> {
    while let Some(parameter) = rest.strip_prefix(';') {
        let parameter = parameter.trim_start_matches(' ');
        let end = parameter
            .find(|c: char| !c.is_ascii_alphanumeric() && !"_-.*".contains(c))
            .unwrap_or(parameter.len());
        if end == 0 {
            return None;
        }

        rest = &parameter[end..];
        if let Some(value) = rest.strip_prefix('=') {
            rest = if value.starts_with('"') {
                unquote(value)?.1
            } else {
                let end = value.find([';', ',', ' ', '\t']).unwrap_or(value.len());
                if end == 0 {
                    return None;
                }
                &value[end..]
            };
        }
    }
    Some(rest)
}

#[cfg(test)]
mod tests {
    use wreq::header::HeaderValue;

    use super::*;

    const ACCEPT_CH: HeaderName = HeaderName::from_static("accept-ch");

    fn accepted(hints: &[&'static str]) -> Accepted {
        hints.iter().copied().map(HeaderName::from_static).collect()
    }

    #[test]
    fn parses_brands_with_separators_in_their_names() {
        assert_eq!(
            parse_brands(r#""Not;A=Brand";v="8", "Chromium";v="150", "Google Chrome";v="150""#),
            Some(vec![
                ("Not;A=Brand".to_owned(), "8".to_owned()),
                ("Chromium".to_owned(), "150".to_owned()),
                ("Google Chrome".to_owned(), "150".to_owned()),
            ])
        );
        assert_eq!(
            parse_brands(r#""Not\"A\\Brand";v="99""#),
            Some(vec![(r#"Not"A\Brand"#.to_owned(), "99".to_owned())])
        );
        for invalid in [
            "",
            r#""Chromium""#,
            r#""Chromium";v="1"#,
            r#""Chromium";v="1","#,
            r#""Chromium";v="1" garbage"#,
            r#""Chromium";v="1" "Google Chrome";v="1""#,
            r#""Not\A";v="1""#,
        ] {
            assert_eq!(parse_brands(invalid), None, "{invalid}");
        }
    }

    #[test]
    fn lists_tokens_and_skips_their_parameters() {
        assert_eq!(tokens(" A, B ,C"), Some(vec!["A", "B", "C"]));
        assert_eq!(tokens("A;x;y=1;z=?0 , B"), Some(vec!["A", "B"]));
        assert_eq!(
            tokens(r#"Sec-CH-UA-Model;x="a, Sec-CH-UA-Arch \" ;, b";y=2, B"#),
            Some(vec!["Sec-CH-UA-Model", "B"])
        );
        assert_eq!(tokens(""), Some(vec![]));
    }

    #[test]
    fn rejects_a_list_that_is_not_made_of_tokens() {
        for value in [
            "A,",
            "A,,B",
            "A B",
            "\"A\"",
            "1",
            "(A B)",
            "A;",
            "A;x=",
            "A;x=\"unterminated",
            "Sec-CH-UA-Arch, ?1",
        ] {
            assert_eq!(tokens(value), None, "{value}");
        }
    }

    #[test]
    fn quotes_printable_ascii() {
        assert_eq!(quote("Pixel 6").as_deref(), Some(r#""Pixel 6""#));
        assert_eq!(quote(r#"a"b\c"#).as_deref(), Some(r#""a\"b\\c""#));
        assert_eq!(quote("").as_deref(), Some(r#""""#));
        assert_eq!(quote("Pixel\n6"), None);
        assert_eq!(quote("Tabl\u{e9}"), None);
    }

    #[test]
    fn lists_hints_across_header_lines() {
        let mut headers = HeaderMap::new();
        assert_eq!(listed(&headers, &ACCEPT_CH), None);

        headers.append(
            ACCEPT_CH,
            HeaderValue::from_static("Sec-CH-UA-Arch, sec-ch-ua-model;x=1 ,UA, a:b"),
        );
        headers.append(
            ACCEPT_CH,
            HeaderValue::from_static("Sec-CH-UA-Arch,Sec-CH-DPR"),
        );
        assert_eq!(
            listed(&headers, &ACCEPT_CH),
            Some(accepted(&[
                "sec-ch-ua-arch",
                "sec-ch-ua-model",
                "ua",
                "sec-ch-dpr"
            ]))
        );

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_CH, HeaderValue::from_static(""));
        assert_eq!(listed(&headers, &ACCEPT_CH), Some(accepted(&[])));
    }

    #[test]
    fn ignores_a_header_that_is_not_a_list_of_tokens() {
        let mut headers = HeaderMap::new();
        headers.append(ACCEPT_CH, HeaderValue::from_static("Sec-CH-UA-Arch"));
        headers.append(ACCEPT_CH, HeaderValue::from_static("Sec-CH-UA Model"));
        assert_eq!(listed(&headers, &ACCEPT_CH), None);

        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT_CH,
            HeaderValue::from_bytes(b"Sec-CH-UA-\xff").unwrap(),
        );
        assert_eq!(listed(&headers, &ACCEPT_CH), None);
    }
}
