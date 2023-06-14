use std::{collections::HashMap, sync::RwLock};

use convert_case::{Boundary, Case, Casing};
use quote::{quote, quote_spanned};
use syn::{LitStr, parse::{Result, Parse, ParseStream}, Token, punctuated::Punctuated, FieldsNamed, Attribute, braced};
use proc_macro2::{TokenStream, Ident, Span};

struct SubcommandData {
    name: Ident,
    attrs: Vec<Attribute>,
    subcommands: Punctuated<Declaration, Token![,]>
}

impl Parse for SubcommandData {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![enum]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let subcommands = content.parse_terminated(Declaration::parse, Token![,])?;

        Ok(Self { name, attrs, subcommands })
    }
}

impl Parse for Declaration {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name = input.parse::<LitStr>()?;
        input.parse::<Token![=>]>()?;
        let fields = input.parse::<FieldsNamed>()?;

        Ok(Self { name, attrs, fields })
    }
}

#[derive(Clone)]
struct Declaration {
    name: LitStr,
    attrs: Vec<Attribute>,
    fields: FieldsNamed,
}

pub(crate) fn expand_subcommand(input: TokenStream) -> Result<TokenStream> {
    let data: SubcommandData = syn::parse2(input)?;

    // Output command structs
    let mut cmd_structs = Vec::new();
    let mut cmd_enums = Vec::new();

    let mut cmd_names = HashMap::new();

    for subcommand in data.subcommands {
        let enum_variant_name = subcommand.name.value().with_boundaries(&[Boundary::Space]).to_case(Case::Pascal);
        cmd_names.insert(subcommand.name.value(), enum_variant_name.clone());
        let name = ident!(&enum_variant_name);

        cmd_enums.push(quote!{
            #name(#name)
        });

        let atts = subcommand.attrs;
        let fields = subcommand.fields;
        cmd_structs.push(quote! {
            #[derive(clap::Parser)]
            #(#atts)*
            pub(crate) struct #name #fields
        });
    }


    let enum_name = data.name;
    let mod_name = &enum_name.to_string().from_case(Case::Title).to_case(Case::Snake);

    // Add data to ALL_DECLARATIONS static, so it can be used in clap-trie proc macro, which needs
    // more information than is provided to expand.
    let res = crate::ALL_DECLARATIONS.get_or_init(|| RwLock::new(HashMap::new()))
        .write().expect("Unable to wrap shared declarations in clap-trie proc macro")
        .insert(enum_name.to_string(), crate::SubcommandEnumDefinition {
            mod_name: mod_name.clone(),
            keys: cmd_names
        });
    if res.is_some() {
        return Ok(quote_spanned!{
            enum_name.span() => compile_error!("trie_subcommand names must be globally unique")
        });
    }

    // Get list of trie keys to commands
    let mod_name = ident!(mod_name);
    let attrs = data.attrs;
    Ok(quote! {
        pub(crate) use #mod_name::#enum_name;
        pub(crate) mod #mod_name {
            #(#attrs)*
            pub(crate) enum #enum_name {
                #(#cmd_enums,)*
            }

            #(#cmd_structs)*
        }
    })
}
