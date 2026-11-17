//! Static `view!` proc macro for Colibrí.
//!
//! The macro parses an HTML-like static view and expands to `colibri_ir` nodes.
//! Dynamic text, dynamic attributes and event bindings are rejected at compile
//! time until the reactive/runtime milestones define their semantics.
//!
//! # Example
//!
//! ```
//! use colibri_macros::view;
//!
//! let node = view! { <p class="note" hidden>"Hola"</p> };
//! assert!(matches!(node, colibri_ir::Node::Fragment { .. }));
//! ```
//!
//! Dynamic content is a compile error until M05/M06:
//!
//! ```compile_fail
//! use colibri_macros::view;
//!
//! let node = view! { <p>{name}</p> };
//! ```

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitStr, Result, Token,
};

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let view = parse_macro_input!(input as View);
    expand(&view).into()
}

struct View {
    nodes: Vec<ViewNode>,
}

enum ViewNode {
    Element(ViewElement),
    Text(LitStr),
}

struct ViewElement {
    tag: String,
    attributes: Vec<ViewAttribute>,
    children: Vec<ViewNode>,
}

struct ViewAttribute {
    name: String,
    value: ViewAttributeValue,
}

enum ViewAttributeValue {
    Boolean,
    Static(LitStr),
}

impl Parse for View {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(parse_node(input)?);
        }
        if nodes.is_empty() {
            return Err(input.error("view! requires at least one node"));
        }
        Ok(Self { nodes })
    }
}

fn parse_node(input: ParseStream) -> Result<ViewNode> {
    if input.peek(Token![<]) {
        if input.peek2(Token![/]) {
            return Err(input.error("unexpected closing tag"));
        }
        parse_element(input)
    } else if input.peek(LitStr) {
        Ok(ViewNode::Text(input.parse()?))
    } else if input.peek(syn::token::Brace) {
        let span = input.span();
        let content;
        braced!(content in input);
        let _: Expr = content.parse()?;
        Err(syn::Error::new(
            span,
            "dynamic text is not supported by the static view! macro yet (M05/M06)",
        ))
    } else {
        Err(input.error("expected an element, a text literal or a closing tag"))
    }
}

fn parse_element(input: ParseStream) -> Result<ViewNode> {
    input.parse::<Token![<]>()?;
    let (tag, tag_span) = parse_name(input)?;
    let mut attributes = Vec::new();

    let self_closing = loop {
        if input.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            break false;
        }
        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            break true;
        }
        attributes.push(parse_attribute(input)?);
    };

    let children = if self_closing || is_void(&tag) {
        Vec::new()
    } else {
        let mut children = Vec::new();
        loop {
            if input.is_empty() {
                return Err(syn::Error::new(
                    tag_span,
                    format!("missing closing tag </{tag}>"),
                ));
            }
            if input.peek(Token![<]) && input.peek2(Token![/]) {
                input.parse::<Token![<]>()?;
                input.parse::<Token![/]>()?;
                let (closing, closing_span) = parse_name(input)?;
                input.parse::<Token![>]>()?;
                if closing != tag {
                    return Err(syn::Error::new(
                        closing_span,
                        format!("expected closing tag </{tag}>, found </{closing}>"),
                    ));
                }
                break;
            }
            children.push(parse_node(input)?);
        }
        children
    };

    Ok(ViewNode::Element(ViewElement {
        tag,
        attributes,
        children,
    }))
}

fn parse_attribute(input: ParseStream) -> Result<ViewAttribute> {
    let (name, _) = parse_name(input)?;

    if input.peek(Token![:]) {
        let span = input.span();
        input.parse::<Token![:]>()?;
        let (suffix, _) = parse_name(input)?;
        return Err(syn::Error::new(
            span,
            format!(
                "`{name}:{suffix}` needs the interactive runtime; static views cannot bind events or namespaced attributes yet (M07)"
            ),
        ));
    }

    let value = if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        if input.peek(LitStr) {
            ViewAttributeValue::Static(input.parse()?)
        } else if input.peek(syn::token::Brace) {
            let span = input.span();
            let content;
            braced!(content in input);
            let _: Expr = content.parse()?;
            return Err(syn::Error::new(
                span,
                "dynamic attribute values are not supported by the static view! macro yet (M05/M06)",
            ));
        } else {
            return Err(input.error("attribute value must be a string literal"));
        }
    } else {
        ViewAttributeValue::Boolean
    };

    Ok(ViewAttribute { name, value })
}

/// Parse `ident ("-" ident)*` so `data-*`, `aria-*` and custom tags work.
/// `parse_any` accepts Rust keywords such as `type` and `for` as HTML names.
fn parse_name(input: ParseStream) -> Result<(String, Span)> {
    let first = input.call(Ident::parse_any)?;
    let span = first.span();
    let mut name = first.to_string();
    while input.peek(Token![-]) {
        input.parse::<Token![-]>()?;
        let part = input.call(Ident::parse_any)?;
        name.push('-');
        name.push_str(&part.to_string());
    }
    Ok((name, span))
}

fn is_void(tag: &str) -> bool {
    VOID_ELEMENTS
        .iter()
        .any(|void| tag.eq_ignore_ascii_case(void))
}

fn expand(view: &View) -> TokenStream2 {
    let children: Vec<TokenStream2> = view.nodes.iter().map(node_tokens).collect();
    let span = span_tokens();
    quote! {
        ::colibri_ir::Node::Fragment {
            children: ::std::vec![#(#children),*],
            span: #span,
        }
    }
}

fn node_tokens(node: &ViewNode) -> TokenStream2 {
    match node {
        ViewNode::Text(literal) => {
            let span = span_tokens();
            quote! {
                ::colibri_ir::Node::Text {
                    value: #literal.to_owned(),
                    span: #span,
                }
            }
        }
        ViewNode::Element(element) => {
            let tag = &element.tag;
            let span = span_tokens();
            let attributes: Vec<TokenStream2> =
                element.attributes.iter().map(attribute_tokens).collect();
            let children: Vec<TokenStream2> = element.children.iter().map(node_tokens).collect();
            quote! {
                ::colibri_ir::Node::Element(::colibri_ir::Element {
                    tag: #tag.to_owned(),
                    attributes: ::std::vec![#(#attributes),*],
                    children: ::std::vec![#(#children),*],
                    span: #span,
                })
            }
        }
    }
}

fn attribute_tokens(attribute: &ViewAttribute) -> TokenStream2 {
    let name = &attribute.name;
    let span = span_tokens();
    match &attribute.value {
        ViewAttributeValue::Boolean => quote! {
            ::colibri_ir::Attribute::Boolean {
                name: #name.to_owned(),
                span: #span,
            }
        },
        ViewAttributeValue::Static(value) => quote! {
            ::colibri_ir::Attribute::Static {
                name: #name.to_owned(),
                value: #value.to_owned(),
                span: #span,
            }
        },
    }
}

/// Stable proc macros cannot read byte offsets, so IR spans are synthetic.
fn span_tokens() -> TokenStream2 {
    quote! {
        ::colibri_ir::SourceSpan {
            file: file!().to_owned(),
            start_byte: 0,
            end_byte: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    fn parse_view(source: &str) -> Result<View> {
        parse_str(source)
    }

    #[test]
    fn parses_static_view() {
        assert!(parse_view("<article class=\"card\" hidden><h1>\"Hola\"</h1></article>").is_ok());
        assert!(parse_view("<input type=\"text\">").is_ok());
    }

    #[test]
    fn rejects_dynamic_text_attributes_and_events() {
        let error = parse_view("<p>{name}</p>")
            .err()
            .expect("expected a syntax error");
        assert!(error.to_string().contains("dynamic text"));

        let error = parse_view("<p class={name}>\"x\"</p>")
            .err()
            .expect("expected a syntax error");
        assert!(error.to_string().contains("dynamic attribute"));

        let error = parse_view("<button on:click={go}>\"x\"</button>")
            .err()
            .expect("expected a syntax error");
        assert!(error.to_string().contains("M07"));
    }

    #[test]
    fn rejects_mismatched_and_missing_closing_tags() {
        let error = parse_view("<div></span>")
            .err()
            .expect("expected a syntax error");
        assert!(error.to_string().contains("expected closing tag </div>"));

        let error = parse_view("<div>\"x\"")
            .err()
            .expect("expected a syntax error");
        assert!(error.to_string().contains("missing closing tag </div>"));
    }

    #[test]
    fn supports_hyphenated_names() {
        let view = parse_view("<p data-id=\"x\" aria-label=\"y\">\"z\"</p>").unwrap();
        let ViewNode::Element(element) = &view.nodes[0] else {
            panic!("expected element");
        };
        assert_eq!(element.attributes[0].name, "data-id");
        assert_eq!(element.attributes[1].name, "aria-label");
    }

    #[test]
    fn requires_at_least_one_node() {
        assert!(parse_view("").is_err());
    }
}
