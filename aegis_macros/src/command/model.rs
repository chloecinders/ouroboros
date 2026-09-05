use syn::spanned::Spanned;
use syn::{Error, Ident, LitChar, LitStr, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Positional,
    Optional,
    Rest,
    Reply,
    ReplyOptional,
    Flag,
}

impl Shape {
    pub fn ident(&self, at: &impl Spanned) -> Ident {
        Ident::new(
            match self {
                Shape::Positional => "Positional",
                Shape::Optional => "Optional",
                Shape::Rest => "Rest",
                Shape::Reply => "Reply",
                Shape::ReplyOptional => "ReplyOptional",
                Shape::Flag => "Flag",
            },
            at.span(),
        )
    }
}

pub struct Modeled {
    pub ident: Ident,
    pub name: String,
    pub inner: Type,
    pub shape: Shape,
    pub optional: bool,
    pub short: Option<LitChar>,
    pub desc: Option<LitStr>,
    pub amend: Ident,
}

pub fn peel<'a>(ty: &'a Type, wrapper: &str) -> Option<&'a Type> {
    let Type::Path(path) = ty else {
        return None;
    };

    let segment = path.path.segments.last()?;

    if segment.ident != wrapper {
        return None;
    }

    let syn::PathArguments::AngleBracketed(generics) = &segment.arguments else {
        return None;
    };

    match generics.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

pub fn model(field: &syn::Field) -> syn::Result<Modeled> {
    let ident = field
        .ident
        .clone()
        .ok_or_else(|| Error::new_spanned(field, "command arguments must be named fields"))?;

    let mut declared = None;
    let mut name = None;
    let mut rest = false;
    let mut reply = false;
    let mut short = None;
    let mut desc = None;
    let mut amend = Ident::new("Never", field.ty.span());

    for attribute in &field.attrs {
        let is_flag = attribute.path().is_ident("flag");

        if !is_flag && !attribute.path().is_ident("arg") {
            continue;
        }

        if declared.is_some() {
            return Err(Error::new_spanned(
                attribute,
                "a field has either #[arg] or #[flag]",
            ));
        }

        declared = Some(is_flag);

        if matches!(attribute.meta, syn::Meta::Path(_)) {
            continue;
        }

        attribute.parse_nested_meta(|option| {
            if option.path.is_ident("rest") {
                rest = true;

                return Ok(());
            }

            if option.path.is_ident("reply") {
                reply = true;

                return Ok(());
            }

            if option.path.is_ident("name") {
                name = Some(option.value()?.parse::<LitStr>()?.value());

                return Ok(());
            }

            if option.path.is_ident("short") {
                short = Some(option.value()?.parse::<LitChar>()?);

                return Ok(());
            }

            if option.path.is_ident("desc") {
                desc = Some(option.value()?.parse::<LitStr>()?);

                return Ok(());
            }

            if option.path.is_ident("amend") {
                amend = option.value()?.parse::<Ident>()?;

                return Ok(());
            }

            Err(option.error("unknown option"))
        })?;
    }

    let Some(is_flag) = declared else {
        return Err(Error::new_spanned(
            field,
            "every command field needs #[arg(...)] or #[flag(...)]",
        ));
    };

    let optional = peel(&field.ty, "Option");
    let carrier = optional.unwrap_or(&field.ty);
    let wrapped = peel(carrier, "Arg");

    if is_flag && desc.is_none() {
        return Err(Error::new_spanned(field, "a flag needs desc = \"...\""));
    }

    if !is_flag && desc.is_some() {
        return Err(Error::new_spanned(field, "desc is for flags only"));
    }

    if is_flag && (rest || reply) {
        return Err(Error::new_spanned(
            &field.ty,
            "rest and reply describe positional arguments",
        ));
    }

    if wrapped.is_some() && !reply {
        return Err(Error::new_spanned(&field.ty, "Arg<T> needs #[arg(reply)]"));
    }

    if is_flag && optional.is_none() && !is_bool(carrier) {
        return Err(Error::new_spanned(
            &field.ty,
            "a flag is a bool or Option<T>",
        ));
    }

    if reply && wrapped.is_none() {
        return Err(Error::new_spanned(
            &field.ty,
            "#[arg(reply)] needs the field typed Arg<T>",
        ));
    }

    let shape = match (is_flag, rest, reply, optional.is_some()) {
        (true, _, _, _) => Shape::Flag,
        (_, true, _, _) => Shape::Rest,
        (_, _, true, false) => Shape::Reply,
        (_, _, true, true) => Shape::ReplyOptional,
        (_, _, _, true) => Shape::Optional,
        _ => Shape::Positional,
    };

    Ok(Modeled {
        name: name.unwrap_or_else(|| ident.to_string()),
        ident,
        inner: wrapped.unwrap_or(carrier).clone(),
        shape,
        optional: optional.is_some(),
        short,
        desc,
        amend,
    })
}

pub fn is_bool(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };

    path.path.is_ident("bool")
}
