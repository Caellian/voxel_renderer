use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Type};

use crate::{
  types::*,
  util::{lit_expr_to_usize, PathCaptureExpr},
};

/// Copied from `wgpu::VertexFormat` from [wgpu](https://docs.rs/wgpu) crate.
#[derive(
  Debug, strum_macros::Display, strum_macros::EnumString, Copy, Clone,
)]
#[repr(u8)]
pub enum VertexFormat {
  Uint8x2 = 0,
  Uint8x4 = 1,
  Sint8x2 = 2,
  Sint8x4 = 3,
  Unorm8x2 = 4,
  Unorm8x4 = 5,
  Snorm8x2 = 6,
  Snorm8x4 = 7,
  Uint16x2 = 8,
  Uint16x4 = 9,
  Sint16x2 = 10,
  Sint16x4 = 11,
  Unorm16x2 = 12,
  Unorm16x4 = 13,
  Snorm16x2 = 14,
  Snorm16x4 = 15,
  Float16x2 = 16,
  Float16x4 = 17,
  Float32 = 18,
  Float32x2 = 19,
  Float32x3 = 20,
  Float32x4 = 21,
  Uint32 = 22,
  Uint32x2 = 23,
  Uint32x3 = 24,
  Uint32x4 = 25,
  Sint32 = 26,
  Sint32x2 = 27,
  Sint32x3 = 28,
  Sint32x4 = 29,
  Float64 = 30,
  Float64x2 = 31,
  Float64x3 = 32,
  Float64x4 = 33,
}

impl VertexFormat {
  fn parse_type(path: syn::Type) -> Self {
    match path {
      Type::Path(tp) => {
        #[cfg(feature = "mint-types")]
        {
          if let Ok(Some(name)) = PathCaptureExpr::parse("mint::?")
            .unwrap()
            .match_on(&tp.path)
          {
            match name.to_lowercase().as_str() {
              // TODO: Add more mint types
              "vector2<f32>" => return VertexFormat::Float32x2,
              "vector3<f32>" => return VertexFormat::Float32x3,
              "vector4<f32>" => return VertexFormat::Float32x4,
              _ => {}
            }
          }
        }
        #[cfg(feature = "glam-types")]
        {
          if let Ok(Some(name)) = PathCaptureExpr::parse("glam::?")
            .unwrap()
            .match_on(&tp.path)
          {
            match name.to_lowercase().as_str() {
              "vec2" => return VertexFormat::Float32x2,
              "vec3" => return VertexFormat::Float32x3,
              "vec4" => return VertexFormat::Float32x4,
              _ => {}
            }
          }
        }
        panic!("unhandled type (enable types through features)")
      }
      Type::Array(arr) => match (*arr.elem, lit_expr_to_usize(&arr.len)) {
        (_, None) => panic!("unable to parse array length (must be a literal)"),
        (syn::Type::Path(path), Some(l)) if l <= 4 => match (
          PrimitiveType::try_from(path)
            .expect("array value must be a primitive"),
          l,
        ) {
          (PrimitiveType::U8, 2) => VertexFormat::Uint8x2,
          (PrimitiveType::U8, 4) => VertexFormat::Uint8x4,
          (PrimitiveType::I8, 2) => VertexFormat::Sint8x2,
          (PrimitiveType::I8, 4) => VertexFormat::Sint8x4,
          (PrimitiveType::U16, 2) => VertexFormat::Uint16x2,
          (PrimitiveType::U16, 4) => VertexFormat::Uint16x4,
          (PrimitiveType::I16, 2) => VertexFormat::Sint16x2,
          (PrimitiveType::I16, 4) => VertexFormat::Sint16x4,
          (PrimitiveType::F16, 2) => VertexFormat::Float16x2,
          (PrimitiveType::F16, 4) => VertexFormat::Float16x4,
          (PrimitiveType::F32, 1) => VertexFormat::Float32,
          (PrimitiveType::F32, 2) => VertexFormat::Float32x2,
          (PrimitiveType::F32, 3) => VertexFormat::Float32x3,
          (PrimitiveType::F32, 4) => VertexFormat::Float32x4,
          (PrimitiveType::U32, 1) => VertexFormat::Uint32,
          (PrimitiveType::U32, 2) => VertexFormat::Uint32x2,
          (PrimitiveType::U32, 3) => VertexFormat::Uint32x3,
          (PrimitiveType::U32, 4) => VertexFormat::Uint32x4,
          (PrimitiveType::I32, 1) => VertexFormat::Sint32,
          (PrimitiveType::I32, 2) => VertexFormat::Sint32x2,
          (PrimitiveType::I32, 3) => VertexFormat::Sint32x3,
          (PrimitiveType::I32, 4) => VertexFormat::Sint32x4,
          (PrimitiveType::F64, 1) => VertexFormat::Float64,
          (PrimitiveType::F64, 2) => VertexFormat::Float64x2,
          (PrimitiveType::F64, 3) => VertexFormat::Float64x3,
          (PrimitiveType::F64, 4) => VertexFormat::Float64x4,
          _ => panic!("unsupported array type"),
        },
        (t, Some(l)) if l <= 4 => panic!("unsupported array type: {:?}", t),
        (_, Some(l)) => panic!("unsupported array lenth: {}", l),
      },
      _ => panic!("unhandled "),
    }
  }

  fn to_norm(self) -> Option<Self> {
    Some(match self {
      VertexFormat::Uint8x2 => VertexFormat::Unorm8x2,
      VertexFormat::Uint8x4 => VertexFormat::Unorm8x4,
      VertexFormat::Sint8x2 => VertexFormat::Snorm8x2,
      VertexFormat::Sint8x4 => VertexFormat::Snorm8x4,
      VertexFormat::Uint16x2 => VertexFormat::Unorm16x2,
      VertexFormat::Uint16x4 => VertexFormat::Unorm16x4,
      VertexFormat::Sint16x2 => VertexFormat::Snorm16x2,
      VertexFormat::Sint16x4 => VertexFormat::Snorm16x4,
      _ => return None,
    })
  }

  fn size(self) -> usize {
    match self {
      VertexFormat::Uint8x2
      | VertexFormat::Sint8x2
      | VertexFormat::Unorm8x2
      | VertexFormat::Snorm8x2 => 2,
      VertexFormat::Uint8x4
      | VertexFormat::Sint8x4
      | VertexFormat::Unorm8x4
      | VertexFormat::Snorm8x4
      | VertexFormat::Uint16x2
      | VertexFormat::Sint16x2
      | VertexFormat::Unorm16x2
      | VertexFormat::Snorm16x2
      | VertexFormat::Float16x2
      | VertexFormat::Float32
      | VertexFormat::Uint32
      | VertexFormat::Sint32 => 4,
      VertexFormat::Uint16x4
      | VertexFormat::Sint16x4
      | VertexFormat::Unorm16x4
      | VertexFormat::Snorm16x4
      | VertexFormat::Float16x4
      | VertexFormat::Float32x2
      | VertexFormat::Uint32x2
      | VertexFormat::Sint32x2
      | VertexFormat::Float64 => 8,
      VertexFormat::Float32x3
      | VertexFormat::Uint32x3
      | VertexFormat::Sint32x3 => 12,
      VertexFormat::Float32x4
      | VertexFormat::Uint32x4
      | VertexFormat::Sint32x4
      | VertexFormat::Float64x2 => 16,
      VertexFormat::Float64x3 => 24,
      VertexFormat::Float64x4 => 32,
    }
  }

  fn to_path(self) -> TokenStream2 {
    let path: syn::Path =
      syn::parse_str(&format!("wgpu::VertexFormat::{}", self.to_string()))
        .unwrap();
    quote! {
      #path
    }
  }
}

impl TryFrom<syn::Path> for VertexFormat {
  type Error = ();
  fn try_from(path: syn::Path) -> Result<Self, Self::Error> {
    if let Ok(Some(format)) = PathCaptureExpr::parse("wgpu::VertexFormat::?")
      .unwrap()
      .match_on(&path)
    {
      Ok(VertexFormat::from_str(&format).map_err(|_| ())?)
    } else {
      Err(())
    }
  }
}

pub struct VertexInfo {
  format: VertexFormat,
  offset: usize,
  shader_location: usize,
  normalized: bool,
}

pub struct CodeGenerator {
  target: syn::Ident,
  step_mode: syn::Path,
  array_stride: Option<usize>,
  fields: Vec<VertexInfo>,
}

impl CodeGenerator {
  fn new(target: syn::Ident) -> Self {
    CodeGenerator {
      target,
      step_mode: syn::parse_str("wgpu::VertexStepMode::Vertex").unwrap(),
      array_stride: None,
      fields: Vec::with_capacity(16),
    }
  }

  pub fn with_attribute(&mut self, a: Attribute) {
    let seg = a.path.get_ident().unwrap();

    match seg.to_string().as_str() {
      "array_stride" => {
        let stride = syn::parse::<syn::LitInt>(a.tokens.into())
          .expect("array_stride value must be an integer");
        self.array_stride = Some(
          stride
            .base10_parse()
            .expect("array_stride value not a valid base 10 integer"),
        );
      }
      "step_mode" => {
        let path = syn::parse::<syn::Path>(a.tokens.into()).expect(
          "step_mode value must be a path to a wgpu::VertexStepMode variant",
        );

        if PathCaptureExpr::parse("wgpu::VertexStepMode::?")
          .unwrap()
          .match_on(&path)
          .is_ok()
        {
          self.step_mode = path.clone();
        } else {
          panic!("step_mode value must be a path to a wgpu::VertexStepMode variant ('VertexStepMode::Vertex' or 'VertexStepMode::Instance')")
        }
      }
      _ => {}
    }
  }

  fn next_free_shader_location(&self) -> usize {
    for i in 0..self.fields.len() {
      if !self.fields.iter().any(|f| f.shader_location == i) {
        return i;
      }
    }

    self.fields.len()
  }

  pub fn with_field(&mut self, f: syn::Field) {
    // infer from type first
    let mut format = VertexFormat::parse_type(f.ty);
    let mut offset = self
      .fields
      .last()
      .map(|f| f.offset + f.format.size())
      .unwrap_or(0);
    let mut shader_location = self.next_free_shader_location();
    let mut normalized = false;

    for a in f.attrs {
      let segments = a.path.segments;
      if segments.len() != 1 {
        continue;
      }
      let name = segments.first().unwrap().ident.to_string();

      match name.as_str() {
        "format" => {
          let path = syn::parse::<syn::Path>(a.tokens.into())
            .expect("expected a VertexFormat variant");

          if let Ok(f) = VertexFormat::try_from(path.clone()) {
            format = f;
          } else {
            panic!("not a valid VertexFormat path")
          }
        }
        "offset" => {
          let value = syn::parse::<syn::LitInt>(a.tokens.into())
            .expect("array_stride value must be an integer  literal");
          offset = value
            .base10_parse()
            .expect("array_stride value not a valid base 10 integer");
        }
        "shader_location" => {
          let value = syn::parse::<syn::LitInt>(a.tokens.into())
            .expect("shader_location value must be an integer literal");
          shader_location = value
            .base10_parse()
            .expect("shader_location value not a valid base 10 integer");
        }
        "norm" | "normalized" => {
          normalized = if a.tokens.is_empty() {
            true
          } else {
            syn::parse::<syn::LitBool>(a.tokens.into())
              .expect("norm value not a boolean literal")
              .value
          };
        }
        _ => {
          // ignore unknown attributes
        }
      }
    }

    self.fields.push(VertexInfo {
      format,
      offset,
      shader_location,
      normalized,
    });
  }

  pub fn generate_token_stream(&self) -> TokenStream {
    let target = self.target.clone();
    let step_mode = self.step_mode.clone();

    let mut layout_fields = Vec::with_capacity(self.fields.len());

    let mut total_size = 0;

    for f in &self.fields {
      let offset = f.offset;
      let shader_location = f.shader_location;
      let format = if f.normalized {
        f.format.to_norm().expect("can't normalize type").to_path()
      } else {
        f.format.to_path()
      };

      layout_fields.push(quote! {wgpu::VertexAttribute {
        offset: #offset as wgpu::BufferAddress,
        shader_location: #shader_location as wgpu::ShaderLocation,
        format: #format
      }});

      total_size += f.format.size();
    }

    let array_stride = match self.array_stride {
      Some(stride) => {
        quote! {#stride}
      }
      None => {
        quote! {std::mem::size_of::<#target>() as wgpu::BufferAddress}
      }
    };

    quote! {
      impl VertexData<'static> for #target {
        const SIZE: usize = #total_size;
        const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[ #(#layout_fields),* ];
        const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
          array_stride: #array_stride,
          step_mode: #step_mode,
          attributes: Self::ATTRIBUTES
        };
      }
    }.into()
  }
}

pub fn process(input: DeriveInput) -> TokenStream {
  let mut gen = CodeGenerator::new(input.ident);

  for attribute in input.attrs {
    gen.with_attribute(attribute);
  }

  let data = match input.data {
    Data::Struct(data) => data,
    // this is because expressing enum variant data repr in consts would be unpractical
    Data::Enum(_) => panic!("enum can't derive VertexData"),
    // this is doable
    Data::Union(_) => unimplemented!("union not supported yet"),
  };

  let fields = match data.fields {
    Fields::Named(fields) => fields.named,
    Fields::Unnamed(fields) => fields.unnamed,
    Fields::Unit => {
      unimplemented!("can't derive VertexData for unit structs")
    }
  };

  for field in fields.into_iter() {
    gen.with_field(field);
  }

  gen.generate_token_stream()
}
