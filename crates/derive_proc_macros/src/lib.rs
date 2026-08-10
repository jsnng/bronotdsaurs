#![no_std]

extern crate alloc;
extern crate proc_macro;

use alloc::vec::Vec;

use proc_macro::TokenStream;
// use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Attribute, Data, DataEnum, DeriveInput, Expr, Ident, Type, parse_macro_input};

fn get_repr_type(ast: &syn::DeriveInput) -> Type {
    for attr in &ast.attrs {
        if attr.path().is_ident("repr")
            && let Ok(ty) = attr.parse_args::<Type>()
        {
            return ty;
        }
    }
    syn::parse_str::<Type>("u8").unwrap()
}

#[proc_macro_derive(TryFromIntoFormat)]
pub fn derive_tryfrom_into_format(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    derive_tryfrom_into_format_impl(ast)
}

fn derive_tryfrom_into_format_impl(ast: DeriveInput) -> TokenStream {
    let data: &DataEnum = match &ast.data {
        Data::Enum(val) => val,
        _ => {
            return syn::Error::new_spanned(&ast.ident, "only enums are supported")
                .to_compile_error()
                .into();
        }
    };

    let ty = get_repr_type(&ast);

    let ident = &ast.ident;
    let cfg_attrs: Vec<&Attribute> = ast
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .collect();

    let variants: Vec<(&Ident, &Expr, Vec<&Attribute>)> = data
        .variants
        .iter()
        .filter(|v| v.discriminant.is_some())
        .map(|v| {
            let cfgs = v
                .attrs
                .iter()
                .filter(|x| x.path().is_ident("cfg"))
                .collect();
            (&v.ident, &v.discriminant.as_ref().unwrap().1, cfgs)
        })
        .collect();

    let count = variants.len();
    let keys: Vec<&Ident> = variants.iter().map(|(x, _, _)| *x).collect();
    let vals: Vec<&Expr> = variants.iter().map(|(_, x, _)| *x).collect();
    let cfgs: Vec<Vec<&Attribute>> = variants.iter().map(|(_, _, z)| z.clone()).collect();

    TokenStream::from(quote! {
        #(#cfg_attrs)*
        impl #ident {
            pub const COUNT: usize = #count;
            /// Zero-allocation lookup — returns `None` for unknown values.
            /// Use this in hot paths. Use `TryFrom` when you need an error message.
            #[inline]
            pub fn from_u8(val: #ty) -> Option<Self> {
                match val {
                    #(
                        #(#cfgs)*
                        #vals => Some(Self::#keys),
                    )*
                    _ => None,
                }
            }
        }
        #(#cfg_attrs)*
        impl TryFrom<#ty> for #ident {
            type Error = DecodeError;
            #[inline]
            fn try_from(val: #ty) -> Result<Self, DecodeError> {
                Self::from_u8(val).ok_or_else(|| DecodeError::InvalidField(format!("TryFrom<{}> unknown value: {:?}", stringify!(#ident), val)))
            }
        }
        #(#cfg_attrs)*
        impl From<#ident> for #ty {
            #[inline]
            fn from(val: #ident) -> Self { val as #ty }
        }
        #(#cfg_attrs)*
        impl core::fmt::Display for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(
                        #(#cfgs)*
                        Self::#keys => write!(f, "{}", stringify!(#keys)),
                    )*
                }
            }
        }

    impl PartialEq<u8> for #ident {
     fn eq(&self, other: &u8) -> bool {
          // Safety: #ident is #[repr(u8)] with only unit variants
          unsafe { *(self as *const Self as *const u8) == *other }
      }
    }
    impl PartialEq<#ident> for u8 {
        fn eq(&self, other: &#ident) -> bool {
          // Safety: #ident is #[repr(u8)] with only unit variants
        unsafe { *self == *(other as *const #ident as *const u8) }
        }
    }
    })
}

#[proc_macro_derive(DefaultDisplayFormat)]
pub fn derive_default_display_format(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    derive_default_display_format_impl(ast)
}

fn derive_default_display_format_impl(ast: DeriveInput) -> TokenStream {
    let data: &DataEnum = match &ast.data {
        Data::Enum(val) => val,
        _ => {
            return syn::Error::new_spanned(&ast.ident, "only enums are supported")
                .to_compile_error()
                .into();
        }
    };

    let ident = &ast.ident;
    let cfg_attrs: Vec<&Attribute> = ast
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .collect();

    let variants: Vec<(&Ident, Vec<&Attribute>)> = data
        .variants
        .iter()
        .map(|v| {
            let cfgs = v
                .attrs
                .iter()
                .filter(|x| x.path().is_ident("cfg"))
                .collect();
            (&v.ident, cfgs)
        })
        .collect();

    let keys: Vec<&Ident> = variants.iter().map(|(x, _)| *x).collect();
    let cfgs: Vec<Vec<&Attribute>> = variants.iter().map(|(_, z)| z.clone()).collect();

    TokenStream::from(quote! {
        #(#cfg_attrs)*
        impl core::fmt::Display for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(
                        #(#cfgs)*
                        Self::#keys => write!(f, "{}", stringify!(#keys)),
                    )*
                }
            }
        }
    })
}
