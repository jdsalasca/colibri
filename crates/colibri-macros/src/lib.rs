//! Proc macros for functional components and typed views.
//!
//! `#[component]` turns a function with typed props into a props struct plus a
//! component function. `view!` builds the shared IR from JSX-like syntax,
//! including dynamic text, dynamic attributes and nested component calls.
//!
//! # Example
//!
//! ```
//! use colibri_ir::Node;
//! use colibri_macros::{component, view};
//! use colibri_ssr::render_static;
//!
//! #[component]
//! fn Greeting(name: String) -> Node {
//!     view! { <p>"Hola " {name}</p> }
//! }
//!
//! let node = view! { <Greeting name="Ada"/> };
//! assert_eq!(render_static(&node).unwrap(), "<p>Hola Ada</p>");
//! ```
//!
//! Events still need the client runtime:
//!
//! ```compile_fail
//! use colibri_macros::view;
//! let node = view! { <button on:click={go}>"x"</button> };
//! ```

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Expr, FnArg, Ident, ItemFn, LitStr, Pat, Result, ReturnType, Token, Type, Visibility,
};

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let view = parse_macro_input!(input as View);
    match expand(&view) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn component(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    match expand_component(function) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct View {
    nodes: Vec<ViewNode>,
}

enum ViewNode {
    Element(ViewElement),
    Text(LitStr),
    DynamicText(Expr),
}

struct ViewElement {
    tag: String,
    tag_span: Span,
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
    Dynamic(Expr),
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
        let content;
        braced!(content in input);
        Ok(ViewNode::DynamicText(content.parse()?))
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
        parse_children(input, &tag, tag_span)?
    };

    if is_component(&tag) && !children.is_empty() {
        return Err(syn::Error::new(
            tag_span,
            "component children are not supported yet; use props",
        ));
    }

    Ok(ViewNode::Element(ViewElement {
        tag,
        tag_span,
        attributes,
        children,
    }))
}

fn parse_children(input: ParseStream, tag: &str, tag_span: Span) -> Result<Vec<ViewNode>> {
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
            return Ok(children);
        }
        children.push(parse_node(input)?);
    }
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
            let content;
            braced!(content in input);
            ViewAttributeValue::Dynamic(content.parse()?)
        } else {
            return Err(input.error("attribute value must be a string literal or {expression}"));
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

fn is_component(tag: &str) -> bool {
    tag.chars().next().is_some_and(char::is_uppercase)
}

fn expand(view: &View) -> Result<TokenStream2> {
    let children = view
        .nodes
        .iter()
        .map(node_tokens)
        .collect::<Result<Vec<_>>>()?;
    let span = span_tokens();
    Ok(quote! {
        ::colibri_ir::Node::Fragment {
            children: ::std::vec![#(#children),*],
            span: #span,
        }
    })
}

fn node_tokens(node: &ViewNode) -> Result<TokenStream2> {
    match node {
        ViewNode::Text(literal) => {
            let span = span_tokens();
            Ok(quote! {
                ::colibri_ir::Node::Text {
                    value: #literal.to_owned(),
                    span: #span,
                }
            })
        }
        ViewNode::DynamicText(expression) => {
            let span = span_tokens();
            Ok(quote! {
                ::colibri_ir::Node::Text {
                    value: format!("{}", (#expression)),
                    span: #span,
                }
            })
        }
        ViewNode::Element(element) if is_component(&element.tag) => component_tokens(element),
        ViewNode::Element(element) => element_tokens(element),
    }
}

fn element_tokens(element: &ViewElement) -> Result<TokenStream2> {
    let tag = &element.tag;
    let span = span_tokens();
    let attributes = element
        .attributes
        .iter()
        .map(attribute_tokens)
        .collect::<Vec<_>>();
    let children = element
        .children
        .iter()
        .map(node_tokens)
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        ::colibri_ir::Node::Element(::colibri_ir::Element {
            tag: #tag.to_owned(),
            attributes: ::std::vec![#(#attributes),*],
            children: ::std::vec![#(#children),*],
            span: #span,
        })
    })
}

fn component_tokens(element: &ViewElement) -> Result<TokenStream2> {
    let tag = syn::parse_str::<Ident>(&element.tag).map_err(|_| {
        syn::Error::new(
            element.tag_span,
            "component name must be a Rust identifier (CamelCase)",
        )
    })?;
    let props = format_ident!("{}Props", tag);
    let fields = element
        .attributes
        .iter()
        .map(|attribute| {
            let field = syn::parse_str::<Ident>(&attribute.name).map_err(|_| {
                syn::Error::new(
                    element.tag_span,
                    "component props must use Rust identifiers, not hyphenated names",
                )
            })?;
            let value = match &attribute.value {
                ViewAttributeValue::Boolean => quote! { true },
                ViewAttributeValue::Static(literal) => quote! { #literal.to_owned() },
                ViewAttributeValue::Dynamic(expression) => quote! { #expression },
            };
            Ok(quote! { #field: #value })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! { #tag(#props { #(#fields),* }) })
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
        ViewAttributeValue::Dynamic(expression) => quote! {
            ::colibri_ir::Attribute::Static {
                name: #name.to_owned(),
                value: format!("{}", (#expression)),
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

fn expand_component(function: ItemFn) -> Result<TokenStream2> {
    validate_component(&function)?;
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = function;
    let name = sig.ident.clone();
    let props_ident = format_ident!("{}Props", name);
    let impl_ident = format_ident!("__colibri_{}", name);
    let docs: Vec<_> = attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
        .collect();
    let component_doc = if docs.is_empty() {
        quote! { #[doc = "Functional component."] }
    } else {
        quote! {}
    };
    let props_doc = format!("Props for `{name}`.");

    let mut fields = Vec::new();
    let mut field_names = Vec::new();
    for argument in &sig.inputs {
        let FnArg::Typed(pat_type) = argument else {
            return Err(syn::Error::new(
                sig.ident.span(),
                "components cannot take self",
            ));
        };
        let Pat::Ident(pat_ident) = &*pat_type.pat else {
            return Err(syn::Error::new(
                pat_type.pat.span(),
                "component props must be simple identifiers",
            ));
        };
        if matches!(*pat_type.ty, Type::Reference(_)) {
            return Err(syn::Error::new(
                pat_type.ty.span(),
                "component props must be owned values, not references",
            ));
        }
        let field = pat_ident.ident.clone();
        let ty = &pat_type.ty;
        fields.push(quote! {
            #[doc = "Prop value."]
            pub #field: #ty
        });
        field_names.push(field);
    }

    let mut impl_sig = sig.clone();
    impl_sig.ident = impl_ident.clone();
    let impl_fn = ItemFn {
        attrs: Vec::new(),
        vis: Visibility::Inherited,
        sig: impl_sig,
        block,
    };
    let output = &sig.output;

    Ok(quote! {
        #(#docs)*
        #[doc = #props_doc]
        #vis struct #props_ident {
            #(#fields),*
        }

        #[allow(non_snake_case)]
        #impl_fn

        #(#attrs)*
        #component_doc
        #[allow(non_snake_case)]
        #vis fn #name(__colibri_props: #props_ident) #output {
            let #props_ident { #(#field_names),* } = __colibri_props;
            #impl_ident(#(#field_names),*)
        }
    })
}

fn validate_component(function: &ItemFn) -> Result<()> {
    let sig = &function.sig;
    if sig.generics.params.iter().next().is_some() {
        return Err(syn::Error::new(
            sig.ident.span(),
            "generic components are not supported yet",
        ));
    }
    if sig.asyncness.is_some() || sig.constness.is_some() || sig.unsafety.is_some() {
        return Err(syn::Error::new(
            sig.ident.span(),
            "async, const and unsafe components are not supported",
        ));
    }
    if matches!(sig.output, ReturnType::Default) {
        return Err(syn::Error::new(
            sig.ident.span(),
            "component must return ::colibri_ir::Node",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    fn parse_view(source: &str) -> Result<View> {
        parse_str(source)
    }

    #[test]
    fn parses_static_and_dynamic_view() {
        assert!(parse_view("<article class=\"card\" hidden><h1>\"Hola\"</h1></article>").is_ok());
        assert!(parse_view("<input type=\"text\">").is_ok());
        assert!(parse_view("<p>{name}</p>").is_ok());
        assert!(parse_view("<p class={name}>\"x\"</p>").is_ok());
    }

    #[test]
    fn rejects_events() {
        let error = parse_view("<button on:click={go}>\"x\"</button>")
            .err()
            .expect("expected a syntax error");
        assert!(error.to_string().contains("M07"));
    }

    #[test]
    fn rejects_component_children() {
        let error = parse_view("<Card>\"x\"</Card>")
            .err()
            .expect("expected a syntax error");
        assert!(error.to_string().contains("children"));
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
