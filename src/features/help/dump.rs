use std::fs;

use serde::Serialize;

use crate::command::registry::Registry;
use crate::command::{CATEGORIES, Category, help};
use crate::features;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("--dump-meta needs a path to write to")]
    Pathless,
    #[error("could not encode the registry: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("could not write the sheet: {0}")]
    Write(#[from] std::io::Error),
}

#[derive(Serialize)]
struct Sheet {
    categories: Vec<String>,
    commands: Vec<Documented>,
}

#[derive(Serialize)]
struct Documented {
    name: &'static str,
    aliases: &'static [&'static str],
    short: &'static str,
    full: &'static str,
    category: String,
    developer: bool,
    hidden: bool,
    syntax: String,
    example: String,
    user: Vec<String>,
    one_of: Vec<String>,
    flags: Vec<Switch>,
}

#[derive(Serialize)]
struct Switch {
    name: &'static str,
    switch: String,
    desc: &'static str,
}

pub fn write(path: &str) -> Result<usize, Failure> {
    let mut registry = Registry::new();

    features::register(&mut registry);

    let sheet = Sheet {
        categories: CATEGORIES.iter().map(Category::to_string).collect(),
        commands: registry
            .all()
            .iter()
            .map(|entry| Documented {
                name: entry.meta.name,
                aliases: entry.meta.aliases,
                short: entry.meta.short,
                full: entry.meta.full,
                category: entry.meta.category.to_string(),
                developer: entry.meta.developer,
                hidden: entry.meta.hidden,
                syntax: entry.syntax(),
                example: entry.example(),
                user: help::permission_names(entry.meta.user),
                one_of: help::permission_names(entry.meta.one_of),
                flags: entry
                    .parameters()
                    .map(|field| Switch {
                        name: field.name,
                        switch: field.switch(),
                        desc: field.desc,
                    })
                    .collect(),
            })
            .collect(),
    };

    fs::write(path, serde_json::to_string_pretty(&sheet)? + "\n")?;

    Ok(sheet.commands.len())
}

pub fn intercept() -> bool {
    let args: Vec<String> = std::env::args().collect();

    let Some(at) = args.iter().position(|arg| arg == "--dump-meta") else {
        return false;
    };

    let dumped = args
        .get(at + 1)
        .ok_or(Failure::Pathless)
        .and_then(|path| write(path));

    match dumped {
        Ok(count) => println!("dumped {count} commands"),
        Err(failure) => {
            eprintln!("{failure}");
            std::process::exit(1);
        }
    }

    true
}
