extern crate proc_macro;

use std::hash::{Hash, Hasher};

use fasthash::FastHasher;
use proc_macro::TokenStream;
use quote::quote;
use static_buffer::StaticBuffer;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod error;
pub(crate) mod static_buffer;
pub(crate) mod types;
pub(crate) mod util;
pub(crate) mod vertex_data;

#[proc_macro_derive(
  VertexData,
  attributes(layout_name, step_mode, array_stride)
)]
pub fn vertex_data(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  vertex_data::process(input)
}

#[proc_macro]
pub fn static_buffer(input: TokenStream) -> TokenStream {
  let mut result = TokenStream::new();

  // TODO: How do I loop this thing?
  let buff: StaticBuffer =
    syn::parse2(input.into()).expect("expected a static buffer");
  result.extend(static_buffer::process(buff));

  result.into()
}

#[proc_macro]
#[allow(non_snake_case)]
pub fn RID(input: TokenStream) -> TokenStream {
  // TODO: How do I loop this thing?
  let value: String = {
    let path: syn::LitStr =
      syn::parse2(input.into()).expect("expected a static buffer");
    path.value()
  };

  let mut hasher = fasthash::city::Hasher128::new();
  value.hash(&mut hasher);
  let hash = hasher.finish();

  quote! {crate::content::resouces::ResourceID(#hash)}.into()
}
