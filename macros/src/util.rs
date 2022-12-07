use crate::error::{CaptureError, InvalidExpression};

pub fn lit_expr_to_usize(expr: &syn::Expr) -> Option<usize> {
  match expr {
    syn::Expr::Lit(syn::ExprLit {
      lit: syn::Lit::Int(int),
      ..
    }) => match int.base10_parse() {
      Ok(value) => Some(value),
      Err(err) => std::panic::panic_any(err),
    },
    _ => None,
  }
}

pub fn path_match_tail<S: AsRef<str>, T: AsRef<[S]>>(
  path: &syn::Path,
  tail: T,
) -> bool {
  tail
    .as_ref()
    .iter()
    .map(|s| s.as_ref())
    .rev()
    .zip(path.segments.iter().rev())
    .all(|(expect, segement)| segement.ident.to_string().as_str() == expect)
}

#[derive(Debug, Clone)]
pub struct PathCaptureExpr {
  wildcart: bool,
  leading_colon: bool,
  segments: Vec<String>,
  capture_last: bool,
}

impl PathCaptureExpr {
  pub fn parse(expr: impl AsRef<str>) -> Result<Self, InvalidExpression> {
    let mut tokens: Vec<&str> = expr.as_ref().split("::").collect();

    let leading_colon = tokens.first().map(|it| *it) == Some("");
    let wildcart = tokens.first().map(|it| *it) == Some("*");
    let mut capture_last = false;
    if leading_colon || wildcart {
      tokens.remove(0);
    }
    if tokens.iter().any(|t| *t == "*") {
      return Err(InvalidExpression::InvalidCapture);
    }
    if tokens.last().map(|it| *it) == Some("?") {
      tokens.pop();
      capture_last = true;
    }

    Ok(PathCaptureExpr {
      wildcart,
      leading_colon,
      segments: tokens.into_iter().map(|t| t.to_string()).collect(),
      capture_last,
    })
  }

  pub fn match_on(
    &self,
    path: &syn::Path,
  ) -> Result<Option<String>, CaptureError> {
    if path.leading_colon.is_some() && !(self.leading_colon || self.wildcart) {
      return Err(CaptureError::UnexpectedLeadingColon);
    }

    let mut segments: Vec<String> =
      path.segments.iter().map(path_segment_to_string).collect();

    let last = if self.capture_last {
      segments.pop()
    } else {
      None
    };

    let match_test = (self.wildcart || self.segments.len() >= segments.len())
      && self
        .segments
        .iter()
        .rev()
        .zip(segments.iter().rev())
        .all(|(a, b)| a == b);

    if match_test {
      Ok(last)
    } else {
      Err(CaptureError::NoMatch(path.clone()))
    }
  }
}

pub fn generic_to_string(a: &syn::AngleBracketedGenericArguments) -> String {
  let args: Vec<String> = a
    .args
    .iter()
    .map(|generic_argument| match generic_argument {
      syn::GenericArgument::Lifetime(_) => {
        panic!("lifetimes not supported")
      }
      syn::GenericArgument::Type(t) => type_to_string(&t),
      syn::GenericArgument::Const(c) => lit_expr_to_usize(c)
        .expect("expected a type or a number")
        .to_string(),
      syn::GenericArgument::Binding(b) => {
        format!("{} = {}", b.ident.to_string(), type_to_string(&b.ty))
      }
      syn::GenericArgument::Constraint(_) => {
        panic!("constraints not supported")
      }
    })
    .collect();

  format!("<{}>", args.join(","))
}

pub fn path_to_string(path: &syn::Path) -> String {
  let leading = if path.leading_colon.is_some() {
    "::"
  } else {
    ""
  };

  let segments: Vec<String> =
    path.segments.iter().map(path_segment_to_string).collect();

  segments.join("::")
}

pub fn path_segment_to_string(seg: &syn::PathSegment) -> String {
  seg.ident.to_string() + &path_args_to_string(&seg.arguments)
}

pub fn path_args_to_string(args: &syn::PathArguments) -> String {
  match args {
    syn::PathArguments::None => String::new(),
    syn::PathArguments::AngleBracketed(a) => generic_to_string(a),
    syn::PathArguments::Parenthesized(_) => {
      panic!("parenthesized path arguments (Fn(A)->B) not supported")
    }
  }
}

pub fn type_to_string(t: &syn::Type) -> String {
  match t {
    syn::Type::Array(arr) => {
      if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(int),
        ..
      }) = &arr.len
      {
        let len: usize = int
          .base10_parse()
          .expect("expected array length to be an integer");
        format!("[{};{}]", type_to_string(&*arr.elem), len)
      } else {
        unimplemented!("unhandled array length expression")
      }
    }
    syn::Type::Group(syn::TypeGroup { elem, .. }) => type_to_string(elem),
    syn::Type::Infer(_) => "_".to_string(),
    syn::Type::Never(_) => "!".to_string(),
    syn::Type::Paren(syn::TypeParen { elem, .. }) => {
      format!("({})", type_to_string(elem))
    }
    syn::Type::Path(path) => path_to_string(&path.path),
    syn::Type::Ptr(ptr) => {
      let kind = if ptr.const_token.is_some() {
        "const"
      } else if ptr.mutability.is_some() {
        "mut"
      } else {
        panic!("pointer neither const nor mut")
      };

      format!("*{} {}", kind, type_to_string(&*ptr.elem))
    }
    syn::Type::Reference(r) => {
      let lifetime = r
        .lifetime
        .as_ref()
        .map(|l| "'".to_owned() + l.ident.to_string().as_str())
        .unwrap_or_default();
      format!("&{} {}", lifetime, type_to_string(&*r.elem))
    }
    syn::Type::Slice(slice) => format!("[{}]", type_to_string(&*slice.elem)),
    syn::Type::Tuple(t) => {
      let ts: Vec<String> = t.elems.iter().map(type_to_string).collect();
      format!("({})", ts.join(","))
    }
    syn::Type::Verbatim(verb) => verb.to_string(),
    _ => panic!("unable to convert type to string"),
  }
}

pub enum StructOrArray {
  Struct(syn::Path),
  Array(syn::TypeArray),
}

impl Into<StructOrArray> for syn::Path {
  fn into(self) -> StructOrArray {
    StructOrArray::Struct(self)
  }
}

impl Into<StructOrArray> for syn::TypeArray {
  fn into(self) -> StructOrArray {
    StructOrArray::Array(self)
  }
}
