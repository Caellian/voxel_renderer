pub enum PrimitiveType {
  Bool,
  U8,
  U16,
  U32,
  U64,
  U128,
  I8,
  I16,
  I32,
  I64,
  I128,
  F16,
  F32,
  F64,
}

#[derive(Debug, thiserror::Error)]
pub enum FieldError {
  #[error("unsupported type: {0}")]
  UnsupportedType(String),
}

impl TryFrom<syn::TypePath> for PrimitiveType {
  type Error = FieldError;

  fn try_from(value: syn::TypePath) -> Result<Self, Self::Error> {
    if value.path.segments.len() == 1 {
      let name = value
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .next()
        .unwrap();
      match name.as_str() {
        "bool" => Ok(PrimitiveType::Bool),
        "u8" => Ok(PrimitiveType::U8),
        "u16" => Ok(PrimitiveType::U16),
        "u32" => Ok(PrimitiveType::U32),
        "u64" => Ok(PrimitiveType::U64),
        "u128" => Ok(PrimitiveType::U128),
        "i8" => Ok(PrimitiveType::I8),
        "i16" => Ok(PrimitiveType::I16),
        "i32" => Ok(PrimitiveType::I32),
        "i64" => Ok(PrimitiveType::I64),
        "i128" => Ok(PrimitiveType::I128),
        "f16" => Ok(PrimitiveType::F16),
        "f32" => Ok(PrimitiveType::F32),
        "f64" => Ok(PrimitiveType::F64),
        _ => Err(FieldError::UnsupportedType(name)),
      }
    } else {
      let path: Vec<String> = value
        .path
        .segments
        .into_iter()
        .map(|seg| seg.ident.to_string())
        .collect();
      let path = path.join("::");
      Err(FieldError::UnsupportedType(path))
    }
  }
}
