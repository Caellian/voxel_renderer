use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, token, Field, Token};

fn not_static() {
  panic!("static buffer attribute only works on static items")
}

pub struct StaticVertex {
  brace: token::Brace,
  fields: Punctuated<syn::FieldValue, Token![,]>,
}

impl Parse for StaticVertex {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    Ok(StaticVertex {
      brace: syn::braced!(content in input),
      fields: content.parse_terminated(syn::FieldValue::parse)?,
    })
  }
}

pub struct StaticBuffer {
  public: Option<Token![pub]>,
  static_lit: Token![static],
  ident: syn::Ident,
  colon: Token![:],
  ty: syn::Type,
  eq: Token![=],
  bracket: token::Bracket,
  fields: Punctuated<StaticVertex, Token![,]>,
}

impl Parse for StaticBuffer {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    Ok(StaticBuffer {
      public: input.parse().expect("an optional pub modifier"),
      static_lit: input.parse().expect("a static keyword"),
      ident: input.parse().expect("buffer name"),
      colon: input.parse().expect("colon"),
      ty: input.parse().expect("a type"),
      eq: input.parse().expect("equals sign"),
      bracket: syn::bracketed!(content in input),
      fields: content
        .parse_terminated(StaticVertex::parse)
        .expect("fields"),
    })
  }
}

// TODO: Load buffer from file
pub fn process(buffer: StaticBuffer) -> TokenStream {
  let mods = if buffer.public.is_some() {
    quote! {pub}
  } else {
    TokenStream2::new()
  };

  let (init_ty, data_ty) = match buffer.ty.clone() {
    syn::Type::Path(tp) => {
      let mut res = tp.path.clone();

      let data_ty = if let Some(seg) = res.segments.last_mut() {
        let result = match seg.arguments {
          syn::PathArguments::AngleBracketed(ref args) => {
            let first = args
              .args
              .iter()
              .next()
              .expect("inner data must be first type argument");
            match first {
              syn::GenericArgument::Type(ty) => ty.clone(),
              _ => panic!("first argument not a type"),
            }
          }
          _ => panic!("first buffer type argument must be inner data"),
        };
        seg.arguments = syn::PathArguments::None;
        result
      } else {
        // sadly we can't infer type
        panic!("first buffer type argument must be inner data")
      };

      (
        syn::Type::Path(syn::TypePath {
          qself: None,
          path: res,
        }),
        data_ty,
      )
    }
    _ => panic!("expected a type path"),
  };

  let values: Vec<TokenStream2> = buffer
    .fields
    .iter()
    .map(|vert| {
      let fields = vert.fields.clone();
      quote! {#data_ty {
          #fields
      }}
    })
    .collect();

  let name = buffer.ident;
  let ty = buffer.ty;

  quote! {#mods static #name: #ty = #init_ty (&[ #(#values),* ])}.into()
}
