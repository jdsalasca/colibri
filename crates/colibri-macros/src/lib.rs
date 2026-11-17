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
    match expand(&view, Backend::Ssr) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Build a `colibri_dom::View` for the browser runtime.
#[proc_macro]
pub fn dom(input: TokenStream) -> TokenStream {
    let view = parse_macro_input!(input as View);
    match expand(&view, Backend::Dom) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Backend {
    Ssr,
    Dom,
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
    span: Span,
    value: ViewAttributeValue,
}

enum ViewAttributeValue {
    Boolean,
    Static(LitStr),
    Dynamic(Expr),
    Event { event: String, handler: Expr },
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
    let (name, span) = parse_name(input)?;

    if input.peek(Token![:]) {
        let colon_span = input.span();
        input.parse::<Token![:]>()?;
        let (suffix, _) = parse_name(input)?;
        if name == "on" {
            input.parse::<Token![=]>()?;
            if !input.peek(syn::token::Brace) {
                return Err(input.error("event handler must be {closure}"));
            }
            let content;
            braced!(content in input);
            return Ok(ViewAttribute {
                name,
                span,
                value: ViewAttributeValue::Event {
                    event: suffix,
                    handler: content.parse()?,
                },
            });
        }
        return Err(syn::Error::new(
            colon_span,
            format!(
                "`{name}:{suffix}` is not supported; only `on:event={{handler}}` is available in dom!"
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

    Ok(ViewAttribute { name, span, value })
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

fn expand(view: &View, backend: Backend) -> Result<TokenStream2> {
    let children = view
        .nodes
        .iter()
        .map(|node| node_tokens(node, backend))
        .collect::<Result<Vec<_>>>()?;
    match backend {
        Backend::Ssr => {
            let span = span_tokens();
            Ok(quote! {
                ::colibri_ir::Node::Fragment {
                    children: ::std::vec![#(#children),*],
                    span: #span,
                }
            })
        }
        Backend::Dom => Ok(quote! {
            ::colibri_dom::View::Fragment(::std::vec![#(#children),*])
        }),
    }
}

fn node_tokens(node: &ViewNode, backend: Backend) -> Result<TokenStream2> {
    match node {
        ViewNode::Text(literal) => match backend {
            Backend::Ssr => {
                let span = span_tokens();
                Ok(quote! {
                    ::colibri_ir::Node::Text {
                        value: #literal.to_owned(),
                        span: #span,
                    }
                })
            }
            Backend::Dom => Ok(quote! {
                ::colibri_dom::View::Text(#literal.to_owned())
            }),
        },
        ViewNode::DynamicText(expression) => match backend {
            Backend::Ssr => {
                let span = span_tokens();
                Ok(quote! {
                    ::colibri_ir::Node::Text {
                        value: format!("{}", (#expression)),
                        span: #span,
                    }
                })
            }
            Backend::Dom => Ok(quote! {
                ::colibri_dom::View::DynamicText(::std::rc::Rc::new(move || format!("{}", (#expression))))
            }),
        },
        ViewNode::Element(element) if element.tag == "Fragment" => {
            fragment_tokens(element, backend)
        }
        ViewNode::Element(element) if element.tag == "Dynamic" => dynamic_tokens(element, backend),
        ViewNode::Element(element) if element.tag == "Keyed" => keyed_tokens(element, backend),
        ViewNode::Element(element) if is_component(&element.tag) => {
            component_tokens(element, backend)
        }
        ViewNode::Element(element) => element_tokens(element, backend),
    }
}

fn dynamic_tokens(element: &ViewElement, backend: Backend) -> Result<TokenStream2> {
    if backend != Backend::Dom {
        return Err(syn::Error::new(
            element.tag_span,
            "Dynamic requires dom! and the client runtime",
        ));
    }
    if !element.children.is_empty() {
        return Err(syn::Error::new(
            element.tag_span,
            "Dynamic does not accept children; use nodes={closure}",
        ));
    }
    let mut nodes = None;
    for attribute in &element.attributes {
        if attribute.name != "nodes" {
            return Err(syn::Error::new(
                element.tag_span,
                "Dynamic only accepts nodes={closure}",
            ));
        }
        let ViewAttributeValue::Dynamic(expression) = &attribute.value else {
            return Err(syn::Error::new(
                element.tag_span,
                "Dynamic nodes must be a closure returning Vec<View>",
            ));
        };
        if nodes.is_some() {
            return Err(syn::Error::new(attribute.span, "duplicate nodes attribute"));
        }
        nodes = Some(expression);
    }
    let nodes = nodes.ok_or_else(|| {
        syn::Error::new(
            element.tag_span,
            "Dynamic requires nodes={closure returning Vec<View>}",
        )
    })?;
    Ok(quote! {
        ::colibri_dom::View::DynamicFragment(::std::rc::Rc::new(#nodes))
    })
}

fn keyed_tokens(element: &ViewElement, backend: Backend) -> Result<TokenStream2> {
    if backend != Backend::Dom {
        return Err(syn::Error::new(
            element.tag_span,
            "Keyed requires dom! and the client runtime",
        ));
    }
    if !element.children.is_empty() {
        return Err(syn::Error::new(
            element.tag_span,
            "Keyed does not accept children; use nodes={closure}",
        ));
    }
    let mut nodes = None;
    for attribute in &element.attributes {
        if attribute.name != "nodes" {
            return Err(syn::Error::new(
                element.tag_span,
                "Keyed only accepts nodes={closure}",
            ));
        }
        let ViewAttributeValue::Dynamic(expression) = &attribute.value else {
            return Err(syn::Error::new(
                element.tag_span,
                "Keyed nodes must be a closure returning Vec<KeyedView>",
            ));
        };
        if nodes.is_some() {
            return Err(syn::Error::new(attribute.span, "duplicate nodes attribute"));
        }
        nodes = Some(expression);
    }
    let nodes = nodes.ok_or_else(|| {
        syn::Error::new(
            element.tag_span,
            "Keyed requires nodes={closure returning Vec<KeyedView>}",
        )
    })?;
    Ok(quote! {
        ::colibri_dom::View::KeyedFragment(::std::rc::Rc::new(#nodes))
    })
}

fn element_tokens(element: &ViewElement, backend: Backend) -> Result<TokenStream2> {
    match backend {
        Backend::Ssr => ssr_element_tokens(element),
        Backend::Dom => dom_element_tokens(element),
    }
}

fn ssr_element_tokens(element: &ViewElement) -> Result<TokenStream2> {
    let tag = &element.tag;
    let span = span_tokens();
    let attributes = element
        .attributes
        .iter()
        .map(|attribute| attribute_tokens(attribute, Backend::Ssr))
        .collect::<Result<Vec<_>>>()?;
    let children = element
        .children
        .iter()
        .map(|child| node_tokens(child, Backend::Ssr))
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

fn dom_element_tokens(element: &ViewElement) -> Result<TokenStream2> {
    let tag = &element.tag;
    let mut attributes = Vec::new();
    let mut events = Vec::new();
    for attribute in &element.attributes {
        match &attribute.value {
            ViewAttributeValue::Event { event, handler } => {
                events.push(quote! {
                    ::colibri_dom::EventBinding {
                        name: #event.to_owned(),
                        handler: ::std::rc::Rc::new(#handler),
                    }
                });
            }
            _ => attributes.push(attribute_tokens(attribute, Backend::Dom)?),
        }
    }
    let children = element
        .children
        .iter()
        .map(|child| node_tokens(child, Backend::Dom))
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        ::colibri_dom::View::Element(::colibri_dom::ElementView {
            tag: #tag.to_owned(),
            attributes: ::std::vec![#(#attributes),*],
            events: ::std::vec![#(#events),*],
            children: ::std::vec![#(#children),*],
        })
    })
}

fn component_tokens(element: &ViewElement, backend: Backend) -> Result<TokenStream2> {
    let tag = syn::parse_str::<Ident>(&element.tag).map_err(|_| {
        syn::Error::new(
            element.tag_span,
            "component name must be a Rust identifier (CamelCase)",
        )
    })?;
    let props = format_ident!("{}Props", tag);
    let mut fields = element
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
                ViewAttributeValue::Event { .. } => {
                    return Err(syn::Error::new(
                        attribute.span,
                        "events on components are not supported; pass a closure prop instead",
                    ));
                }
            };
            Ok(quote! { #field: #value })
        })
        .collect::<Result<Vec<_>>>()?;
    if !element.children.is_empty() {
        let children = element
            .children
            .iter()
            .map(|child| node_tokens(child, backend))
            .collect::<Result<Vec<_>>>()?;
        fields.push(quote! { children: ::std::vec![#(#children),*] });
    }
    Ok(quote! { #tag(#props { #(#fields),* }) })
}

fn fragment_tokens(element: &ViewElement, backend: Backend) -> Result<TokenStream2> {
    if !element.children.is_empty() {
        return Err(syn::Error::new(
            element.tag_span,
            "Fragment does not accept children; use nodes={vec}",
        ));
    }
    let mut nodes = None;
    for attribute in &element.attributes {
        if attribute.name != "nodes" {
            return Err(syn::Error::new(
                element.tag_span,
                "Fragment only accepts nodes={expression}",
            ));
        }
        let ViewAttributeValue::Dynamic(expression) = &attribute.value else {
            return Err(syn::Error::new(
                element.tag_span,
                "Fragment nodes must be a {expression}",
            ));
        };
        if nodes.is_some() {
            return Err(syn::Error::new(attribute.span, "duplicate nodes attribute"));
        }
        nodes = Some(expression);
    }
    match backend {
        Backend::Ssr => {
            let span = span_tokens();
            match nodes {
                Some(nodes) => Ok(quote! {
                    ::colibri_ir::Node::Fragment {
                        children: #nodes,
                        span: #span,
                    }
                }),
                None => Ok(quote! {
                    ::colibri_ir::Node::Fragment {
                        children: ::std::vec![],
                        span: #span,
                    }
                }),
            }
        }
        Backend::Dom => match nodes {
            Some(nodes) => Ok(quote! { ::colibri_dom::View::Fragment(#nodes) }),
            None => Ok(quote! { ::colibri_dom::View::Fragment(::std::vec![]) }),
        },
    }
}

fn attribute_tokens(attribute: &ViewAttribute, backend: Backend) -> Result<TokenStream2> {
    let name = &attribute.name;
    match &attribute.value {
        ViewAttributeValue::Boolean => Ok(match backend {
            Backend::Ssr => {
                let span = span_tokens();
                quote! {
                    ::colibri_ir::Attribute::Boolean {
                        name: #name.to_owned(),
                        span: #span,
                    }
                }
            }
            Backend::Dom => quote! {
                ::colibri_dom::Attribute::Boolean {
                    name: #name.to_owned(),
                }
            },
        }),
        ViewAttributeValue::Static(value) => Ok(match backend {
            Backend::Ssr => {
                let span = span_tokens();
                quote! {
                    ::colibri_ir::Attribute::Static {
                        name: #name.to_owned(),
                        value: #value.to_owned(),
                        span: #span,
                    }
                }
            }
            Backend::Dom => quote! {
                ::colibri_dom::Attribute::Static {
                    name: #name.to_owned(),
                    value: #value.to_owned(),
                }
            },
        }),
        ViewAttributeValue::Dynamic(expression) => Ok(match backend {
            Backend::Ssr => {
                let span = span_tokens();
                quote! {
                    ::colibri_ir::Attribute::Static {
                        name: #name.to_owned(),
                        value: format!("{}", (#expression)),
                        span: #span,
                    }
                }
            }
            Backend::Dom => quote! {
                ::colibri_dom::Attribute::Dynamic {
                    name: #name.to_owned(),
                    value: ::std::rc::Rc::new(move || format!("{}", (#expression))),
                }
            },
        }),
        ViewAttributeValue::Event { .. } => Err(syn::Error::new(
            attribute.span,
            "events require dom! and the client runtime",
        )),
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
    let attach = if returns_view(output) {
        quote! { ::colibri_dom::attach_effects(__colibri_value, __colibri_effects) }
    } else {
        quote! {{
            drop(__colibri_effects);
            __colibri_value
        }}
    };

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
            let (__colibri_value, __colibri_effects) =
                ::colibri_core::with_effect_scope(|| {
                    let #props_ident { #(#field_names),* } = __colibri_props;
                    #impl_ident(#(#field_names),*)
                });
            #attach
        }
    })
}

fn returns_view(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    matches!(
        &**ty,
        Type::Path(path)
            if path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "View")
    )
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
    fn parses_events() {
        let view = parse_view("<button on:click={go}>\"x\"</button>").unwrap();
        let ViewNode::Element(element) = &view.nodes[0] else {
            panic!("expected element");
        };
        assert!(matches!(
            element.attributes[0].value,
            ViewAttributeValue::Event { .. }
        ));
    }

    #[test]
    fn parses_component_children() {
        assert!(parse_view("<Card>\"x\"</Card>").is_ok());
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
