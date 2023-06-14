use proc_macro2::{TokenStream, Ident, Span};
use quote::{quote, quote_spanned};
use syn::{parse::{Result, Parse, ParseStream}, punctuated::Punctuated, Token, braced, Path, LitStr, PathSegment, PathArguments, Attribute};

use crate::{Trie, TrieKey};

struct ClapTrieData {
    name: Ident,
    attrs: Vec<Attribute>,
    enum_paths: Punctuated<Path, Token![,]>,
}

impl Parse for ClapTrieData {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![enum]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let enum_paths = content.parse_terminated(Path::parse, Token![,])?;

        Ok(Self {
            name,
            attrs,
            enum_paths,
        })
    }
}

struct TrieItem {
    struct_name: Ident,
    struct_path: Path,
    enum_path: Path,
    enum_name: Ident,
}

pub(crate) fn expand_trie(input: TokenStream) -> Result<TokenStream> {
    let data: ClapTrieData = syn::parse2(input)?;
    let read_lock = crate::ALL_DECLARATIONS
        .get().expect("The clap_trie macro must be called after all clap_subcommand definitions")
        .read().expect("Unable to read lock in clap_trie subcommand");

    let mut trie = Trie::new();
    let mut enum_variants = Vec::new();

    for enum_path in data.enum_paths {
        let enum_name = &enum_path.segments.last().unwrap().ident;
        let Some(declarations) = read_lock.get(&enum_name.to_string()) else {
            return Ok(quote_spanned!{
                enum_name.span() => compile_error!("Invalid trie subcommand reference '{}'. Are you sure it has been defined?", enum_name.to_string())
            });
        };

        enum_variants.push(quote!(#enum_name(#enum_path)));

        // Replace last ident in enum_path with the module name generated in expand_subcommand macro
        let mod_path = change_path_ident!(enum_path, ident!(&declarations.mod_name));

        // Create trie
        for (key, struct_name) in &declarations.keys {
            let struct_name = ident!(struct_name);
            let struct_path = append_path!(mod_path, struct_name.clone());
            let res = trie.insert(key, TrieItem {
                struct_name,
                struct_path,
                enum_path: enum_path.clone(),
                enum_name: enum_name.clone(),
            });
            if res.is_some() {
                println!("Duplicate variant: {}", key);
                return Ok(quote_spanned!{
                    data.name.span() => compile_error!("All trie command enum variants must be unique")
                });
            }
        }
    }

    // Visit command trie from bottom up. Creating each step of the recursion on the way
    let from_arg_aggregate = trie.aggregate_depth_first_root(&mut from_arg_aggregate);
    let subcommand_aggregate = trie.aggregate_depth_first(&mut subcommand_aggregate);
    let subcommand_names = trie.child_keys("").unwrap();

    let attrs = data.attrs;
    let name = data.name;
    Ok(quote!{
        #(#attrs)*
        enum #name {
            #(#enum_variants),*
        }

        impl clap::FromArgMatches for #name {
            fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
                match Some(("", matches)) {
                    #from_arg_aggregate
                    _ => unreachable!()
                }
                Err(clap::Error::new(clap::error::ErrorKind::InvalidSubcommand))
            }

            fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
                unimplemented!("update_from_arg_matches")
            }
        }

        impl clap::Subcommand for #name {
            fn augment_subcommands(cmd: clap::Command) -> clap::Command {
                cmd.subcommands([
                    #(#subcommand_aggregate),*
                ])
            }

            fn augment_subcommands_for_update(cmd: clap::Command) -> clap::Command {
                unimplemented!("augment_subcommands_for_update")
            }

            fn has_subcommand(name: &str) -> bool {
                match name {
                    #(#subcommand_names => true,)*
                    _ => false,
                }
            }
        }
    })
}

fn from_arg_aggregate(value: Option<&mut TrieItem>, key: String, memo: Vec<TokenStream>) -> TokenStream {
    let return_no_subcommand = if let Some(value) = value {
        let TrieItem { struct_path, enum_path, enum_name, struct_name: enum_variant_name } = value;
        quote! {
            return Ok(Self::#enum_name(#enum_path::#enum_variant_name(<#struct_path as clap::FromArgMatches>::from_arg_matches(matches)?)));
        }
    } else {
        quote! {
            return Err(clap::Error::new(clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand));
        }
    };

    let key = key.key().unwrap_or_default();
    quote!{
        Some((#key, matches)) => {
            match matches.subcommand() {
                None => { #return_no_subcommand }
                #(#memo)*
                Some(_) => {}, // Fall through, end of function returns InvalidSubcommand
            }
        }
    }
}

fn subcommand_aggregate(value: Option<&mut TrieItem>, key: String, memo: Vec<TokenStream>) -> TokenStream {
    let name = LitStr::new(key.key().unwrap(), Span::call_site());
    let mut command = quote!(clap::Command::new(#name));

    // Use CommandFactory::augment_args if a command struct exists
    if let Some(TrieItem { struct_path, .. }) = value {
        command = quote!{
            <#struct_path as clap::Args>::augment_args(#command)
        };
    }

    // If there are children aggregates, add them as subcommands
    if ! memo.is_empty() {
        command.extend(quote!{
            .args_conflicts_with_subcommands(true) // Don't allow args on intermediate commands
            .subcommands([
                #(#memo),*
            ])
        });
    }

    command
}
