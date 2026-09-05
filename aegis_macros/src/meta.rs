use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Ident, LitBool, LitStr, Token, bracketed};

#[derive(Default)]
struct Block {
    name: Option<LitStr>,
    aliases: Vec<LitStr>,
    short: Option<LitStr>,
    full: Option<LitStr>,
    category: Option<Ident>,
    user: Vec<Ident>,
    one_of: Vec<Ident>,
    bot: Vec<Ident>,
    developer: Option<LitBool>,
    hidden: Option<LitBool>,
    edit: Option<Ident>,
}

fn list<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let inner;

    bracketed!(inner in input);

    Ok(Punctuated::<T, Token![,]>::parse_terminated(&inner)?
        .into_iter()
        .collect())
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut block = Block::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => block.name = Some(input.parse()?),
                "aliases" => block.aliases = list(input)?,
                "short" => block.short = Some(input.parse()?),
                "full" => block.full = Some(input.parse()?),
                "category" => block.category = Some(input.parse()?),
                "user" => block.user = list(input)?,
                "one_of" => block.one_of = list(input)?,
                "bot" => block.bot = list(input)?,
                "developer" => block.developer = Some(input.parse()?),
                "hidden" => block.hidden = Some(input.parse()?),
                "edit" => block.edit = Some(input.parse()?),
                unknown => {
                    return Err(Error::new(key.span(), format!("unknown key `{unknown}`")));
                }
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        for (value, missing) in [
            (block.name.is_none(), "name"),
            (block.short.is_none(), "short"),
            (block.full.is_none(), "full"),
            (block.category.is_none(), "category"),
        ] {
            if value {
                return Err(Error::new(
                    input.span(),
                    format!("meta! is missing `{missing}`"),
                ));
            }
        }

        Ok(block)
    }
}

fn permissions(names: &[Ident], baseline: bool) -> impl ToTokens {
    if names.is_empty() && !baseline {
        return quote! { ::serenity::all::Permissions::empty() };
    }

    let seed = match baseline {
        true => quote! { crate::command::BASELINE.bits() },
        false => quote! { 0 },
    };

    quote! {
        ::serenity::all::Permissions::from_bits_truncate(
            #seed #(| ::serenity::all::Permissions::#names.bits())*
        )
    }
}

pub fn expand(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let block: Block = match syn::parse(item) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error().into(),
    };

    let name = block.name;
    let aliases = block.aliases;
    let short = block.short;
    let full = block.full;
    let category = block.category;
    let user = permissions(&block.user, false);
    let one_of = permissions(&block.one_of, false);
    let bot = permissions(&block.bot, true);
    let developer = block
        .developer
        .map(|flag| quote! { #flag })
        .unwrap_or_else(|| quote! { false });
    let hidden = block
        .hidden
        .map(|flag| quote! { #flag })
        .unwrap_or_else(|| quote! { false });
    let edit = block
        .edit
        .unwrap_or_else(|| Ident::new("Fixed", proc_macro::Span::call_site().into()));

    quote! {
        crate::command::Meta {
            name: #name,
            aliases: &[#(#aliases),*],
            short: #short,
            full: #full,
            category: crate::command::Category::#category,
            user: #user,
            one_of: #one_of,
            bot: #bot,
            developer: #developer,
            hidden: #hidden,
            edit: crate::command::EditMode::#edit,
        }
    }
    .into()
}
