use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Error;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Field, Ident};

#[proc_macro_derive(TracerOpts)]
pub fn tracer_opts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_tracer_opts(&input) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn impl_tracer_opts(ast: &syn::DeriveInput) -> Result<TokenStream, Error> {
    let data = match &ast.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let ident = &ast.ident;
                let mut methods = Vec::new();
                for field in fields.named.iter() {
                    let ident = field.ident.as_ref().ok_or_else(|| {
                        Error::new(
                            ast.span(),
                            "TracerOpts is not supported fields of tuple structs",
                        )
                    })?;
                    let ty = extract_opt_type(&field).ok_or_else(|| {
                        Error::new(ast.span(), "TracerOpts supports optional fields only")
                    })?;
                    methods.push(quote! {
                        pub fn #ident(mut self, value: impl Into<#ty>) -> Self {
                            self.#ident = Some(value.into());
                            self
                        }
                    });
                    let set_ident = Ident::new(&format!("set_{}", ident), Span::call_site());
                    methods.push(quote! {
                        pub fn #set_ident(&mut self, value: impl Into<#ty>) -> &mut Self {
                            self.#ident = Some(value.into());
                            self
                        }
                    });
                }
                quote! {
                    impl #ident {
                        #( #methods )*
                    }
                }
            }
            syn::Fields::Unnamed(_) => {
                return Err(Error::new(
                    ast.span(),
                    "TracerOpts is not supported for tuple structs",
                ))
            }
            syn::Fields::Unit => {
                return Err(Error::new(
                    ast.span(),
                    "TracerOpts is not supported for unit structs",
                ))
            }
        },
        syn::Data::Enum(_) => {
            return Err(Error::new(
                ast.span(),
                "TracerOpts is not supported for enums",
            ))
        }
        syn::Data::Union(_) => {
            return Err(Error::new(
                ast.span(),
                "TracerOpts is not supported for unions",
            ))
        }
    };
    Ok(data.into())
}

fn extract_opt_type(field: &Field) -> Option<&syn::Type> {
    let path = if let syn::Type::Path(type_path) = &field.ty {
        if type_path.qself.is_some() {
            return None;
        } else {
            &type_path.path
        }
    } else {
        return None;
    };
    let segment = path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let generic_params =
        if let syn::PathArguments::AngleBracketed(generic_params) = &segment.arguments {
            generic_params
        } else {
            return None;
        };
    if let syn::GenericArgument::Type(ty) = generic_params.args.first()? {
        Some(ty)
    } else {
        None
    }
}
