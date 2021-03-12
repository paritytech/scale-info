// TODO: copyright and license
use syn::{self, Meta::{List, NameValue}, NestedMeta::{Lit, Meta}};
use syn::parse::{self, Parse};
use proc_macro2::TokenStream;
use quote::ToTokens;
use super::symbol::{Symbol, BOUND, SCALE_INFO};
use super::Ctxt;
use super::respan::respan;

struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    #[allow(unused)]
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        Attr {
            cx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            self.cx
                .error_spanned_by(tokens, format!("duplicate scale-info attribute `{}`", self.name));
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    #[allow(unused)]
    fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(obj, value);
        }
    }

    #[allow(unused)]
    fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    fn get(self) -> Option<T> {
        self.value
    }

    #[allow(unused)]
    fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(v) => Some((self.tokens, v)),
            None => None,
        }
    }
}


#[derive(Debug)]
/// Represents struct or enum attribute information.
pub struct Container {
    name: String, // TODO: not sure what this `Name` even is in serde
    bound: Option<Vec<syn::WherePredicate>>,
}

impl Container {
    /// Extract the `#[scale_info(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, item: &syn::DeriveInput) -> Self {
        let mut bound = Attr::none(cx, BOUND);

        for meta_item in item
            .attrs
            .iter()
            .flat_map(|attr| get_scale_info_meta_items(cx, attr))
            .flatten()
        {
            match &meta_item {
                // Parse `#[scale_info(bound = "T: SomeBound")]`
                Meta(NameValue(m)) if m.path == BOUND => {
                    if let Ok(where_predicates) = parse_lit_into_where(cx, BOUND, BOUND, &m.lit) {
                        bound.set(&m.path, where_predicates.clone());
                    }
                }

                Meta(meta_item) => {
                    let path = meta_item
                        .path()
                        .into_token_stream()
                        .to_string()
                        .replace(' ', "");
                    cx.error_spanned_by(
                        meta_item.path(),
                        format!("unknown scale-info container attribute `{}`", path),
                    );
                }

                Lit(lit) => {
                    cx.error_spanned_by(lit, "unexpected literal in scale-info container attribute");
                }
            }
        }

        Container {
            name: item.ident.to_string(),
            bound: bound.get(),
        }
    }

    pub fn bound(&self) -> Option<&[syn::WherePredicate]> {
        self.bound.as_ref().map(|vec| &vec[..])
    }
}

pub fn get_scale_info_meta_items(cx: &Ctxt, attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, ()> {
    if attr.path != SCALE_INFO {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(other) => {
            cx.error_spanned_by(other, "expected #[scale_info(...)]");
            Err(())
        }
        Err(err) => {
            cx.syn_error(err);
            Err(())
        }
    }
}

fn parse_lit_into_where(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    lit: &syn::Lit,
) -> Result<Vec<syn::WherePredicate>, ()> {
    let string = get_lit_str2(cx, attr_name, meta_item_name, lit)?;
    if string.value().is_empty() {
        return Ok(Vec::new());
    }

    let where_string = syn::LitStr::new(&format!("where {}", string.value()), string.span());

    parse_lit_str::<syn::WhereClause>(&where_string)
        .map(|wh| wh.predicates.into_iter().collect())
        .map_err(|err| cx.error_spanned_by(lit, err))
}

fn get_lit_str2<'a>(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected scale_info {} attribute to be a string: `{} = \"...\"`",
                attr_name, meta_item_name
            ),
        );
        Err(())
    }
}

fn parse_lit_str<T>(s: &syn::LitStr) -> parse::Result<T>
where
    T: Parse,
{
    let tokens = spanned_tokens(s)?;
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> parse::Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan(stream, s.span()))
}
