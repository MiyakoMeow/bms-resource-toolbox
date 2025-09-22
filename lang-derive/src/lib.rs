use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Ident, LitStr, Result, Token, parse_macro_input};

struct LangArgs {
    name: LitStr,
    desc: LitStr,
}

impl Parse for LangArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<LitStr> = None;
        let mut desc: Option<LitStr> = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            if key == "name" {
                name = Some(value);
            } else if key == "desc" {
                desc = Some(value);
            } else {
                return Err(syn::Error::new_spanned(
                    key,
                    "unknown key; expected `name` or `desc`",
                ));
            }
            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            }
        }
        Ok(LangArgs {
            name: name.ok_or_else(|| syn::Error::new(input.span(), "missing `name`"))?,
            desc: desc.ok_or_else(|| syn::Error::new(input.span(), "missing `desc`"))?,
        })
    }
}

#[proc_macro_derive(Localized, attributes(lang_chinese, lang_english))]
pub fn derive_localized(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new(enum_ident.span(), "Localized can only be derived for enums")
            .to_compile_error()
            .into();
    };

    let mut zh_arms = Vec::new();
    let mut en_arms = Vec::new();

    for variant in data_enum.variants.iter() {
        let v_ident = &variant.ident;
        let pat = match &variant.fields {
            syn::Fields::Unit => quote! { Self::#v_ident },
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => quote! { Self::#v_ident { .. } },
        };
        let mut zh: Option<LangArgs> = None;
        let mut en: Option<LangArgs> = None;
        for attr in &variant.attrs {
            if attr.path().is_ident("lang_chinese") {
                let args = match attr.parse_args::<LangArgs>() {
                    Ok(a) => a,
                    Err(e) => return e.to_compile_error().into(),
                };
                zh = Some(args);
            } else if attr.path().is_ident("lang_english") {
                let args = match attr.parse_args::<LangArgs>() {
                    Ok(a) => a,
                    Err(e) => return e.to_compile_error().into(),
                };
                en = Some(args);
            }
        }
        let zh = match zh {
            Some(z) => z,
            None => {
                return syn::Error::new_spanned(
                    v_ident,
                    "missing #[lang_chinese(name = ..., desc = ...)]",
                )
                .to_compile_error()
                .into();
            }
        };
        let en = match en {
            Some(e) => e,
            None => {
                return syn::Error::new_spanned(
                    v_ident,
                    "missing #[lang_english(name = ..., desc = ...)]",
                )
                .to_compile_error()
                .into();
            }
        };

        let zh_name = zh.name;
        let zh_desc = zh.desc;
        let en_name = en.name;
        let en_desc = en.desc;
        zh_arms.push(quote! { #pat => lang_core::LangText { name: #zh_name, desc: #zh_desc } });
        en_arms.push(quote! { #pat => lang_core::LangText { name: #en_name, desc: #en_desc } });
    }

    let expanded = quote! {
        impl lang_core::Localized for #enum_ident {
            fn zh(&self) -> lang_core::LangText {
                match self {
                    #(#zh_arms,)*
                }
            }
            fn en(&self) -> lang_core::LangText {
                match self {
                    #(#en_arms,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
