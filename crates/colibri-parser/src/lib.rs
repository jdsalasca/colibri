//! A deliberately limited single-file component block parser.
//! This is not a full HTML parser and does not validate Rust/SCSS syntax.
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBlock {
    pub content: String,
    pub lang: Option<String>,
    pub scoped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleFileComponent {
    pub script: Option<String>,
    pub template: String,
    pub style: Option<StyleBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidTopLevel,
    UnsupportedTag(String),
    MissingClosingTag(String),
    DuplicateBlock(String),
    MissingTemplate,
    InvalidScriptLanguage,
    InvalidStyleLanguage,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTopLevel => write!(f, "invalid top-level content or malformed block"),
            Self::UnsupportedTag(tag) => write!(f, "unsupported top-level block: {tag}"),
            Self::MissingClosingTag(tag) => write!(f, "missing closing tag: {tag}"),
            Self::DuplicateBlock(tag) => write!(f, "duplicate block: {tag}"),
            Self::MissingTemplate => write!(f, "missing non-empty <template>"),
            Self::InvalidScriptLanguage => write!(f, "script must use lang=\"rust\""),
            Self::InvalidStyleLanguage => write!(f, "style language must be css or scss"),
        }
    }
}

impl Error for ParseError {}

fn attr_value(open: &str, name: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let needle = format!("{name}={quote}");
        if let Some(index) = open.find(&needle) {
            let after = &open[index + needle.len()..];
            return after.find(quote).map(|end| after[..end].to_owned());
        }
    }
    None
}

fn is_scoped(open: &str) -> bool {
    open.split_ascii_whitespace()
        .any(|token| token == "scoped" || token.starts_with("scoped="))
}

pub fn parse(source: &str) -> Result<SingleFileComponent, ParseError> {
    let mut rest = source;
    let mut script = None;
    let mut template = None;
    let mut style = None;
    loop {
        rest = rest.trim_start();
        if rest.starts_with("<!--") {
            let end = rest.find("-->").ok_or(ParseError::InvalidTopLevel)?;
            rest = &rest[end + 3..];
            continue;
        }
        if rest.is_empty() {
            break;
        }
        if !rest.starts_with('<') {
            return Err(ParseError::InvalidTopLevel);
        }
        let end = rest.find('>').ok_or(ParseError::InvalidTopLevel)?;
        let open = &rest[1..end];
        if open.starts_with('/') || open.ends_with('/') {
            return Err(ParseError::InvalidTopLevel);
        }
        let tag = open.split_ascii_whitespace().next().unwrap_or("");
        if !["script", "template", "style"].contains(&tag) {
            return Err(ParseError::UnsupportedTag(tag.to_string()));
        }
        let close = format!("</{tag}>");
        let following = &rest[end + 1..];
        let closing = following.find(&close)
            .ok_or_else(|| ParseError::MissingClosingTag(tag.into()))?;
        let body = following[..closing].to_owned();
        rest = &following[closing + close.len()..];
        match tag {
            "script" => {
                if script.is_some() {
                    return Err(ParseError::DuplicateBlock(tag.into()));
                }
                if attr_value(open, "lang").as_deref() != Some("rust") {
                    return Err(ParseError::InvalidScriptLanguage);
                }
                script = Some(body);
            }
            "template" => {
                if template.is_some() {
                    return Err(ParseError::DuplicateBlock(tag.into()));
                }
                template = Some(body);
            }
            "style" => {
                if style.is_some() {
                    return Err(ParseError::DuplicateBlock(tag.into()));
                }
                let lang = attr_value(open, "lang");
                if !matches!(lang.as_deref(), None | Some("css") | Some("scss")) {
                    return Err(ParseError::InvalidStyleLanguage);
                }
                style = Some(StyleBlock { content: body, lang, scoped: is_scoped(open) });
            }
            _ => unreachable!(),
        }
    }
    let template = template.ok_or(ParseError::MissingTemplate)?;
    if template.trim().is_empty() {
        return Err(ParseError::MissingTemplate);
    }
    Ok(SingleFileComponent { script, template, style })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sfc_blocks() {
        let sfc = parse(
            "<style scoped lang=\"scss\">button{color:red}</style>\n\
             <script lang=\"rust\">let x = 1;</script>\n\
             <template><button>{x}</button></template>",
        )
        .unwrap();
        assert!(sfc.script.unwrap().contains("let x"));
        assert!(sfc.template.contains("<button>"));
        assert!(sfc.style.unwrap().scoped);
    }

    #[test]
    fn detects_duplicate_and_missing_templates() {
        assert_eq!(parse("<script lang=\"rust\">x</script>"), Err(ParseError::MissingTemplate));
        assert_eq!(
            parse("<template>a</template><template>b</template>"),
            Err(ParseError::DuplicateBlock("template".into()))
        );
    }

    #[test]
    fn rejects_unsupported_languages() {
        assert_eq!(
            parse("<script lang=\"js\">x</script><template>ok</template>"),
            Err(ParseError::InvalidScriptLanguage)
        );
    }
}
