use std::collections::HashMap;
use std::iter::once;

use crate::command::args::Field;
use crate::command::cx::Cx;
use crate::command::error::Result;
use crate::command::stream::Stream;
use crate::command::{Boxed, Category, Command, Meta, Response};

pub type Execute = for<'a> fn(&'a mut Cx, &'a mut Stream) -> Boxed<'a, Result<Response>>;
pub type Rehearse = for<'a> fn(&'a mut Cx, &'a mut Stream) -> Boxed<'a, Result<serde_json::Value>>;

#[derive(Clone, Copy)]
pub struct Entry {
    pub meta: Meta,
    pub fields: &'static [Field],
    pub execute: Execute,
    pub rehearse: Rehearse,
}

impl Entry {
    pub fn of<C: Command>() -> Self {
        Self {
            meta: C::META,
            fields: C::FIELDS,
            execute: dispatch::<C>,
            rehearse: rehearse::<C>,
        }
    }

    pub fn syntax(&self) -> String {
        self.fields
            .iter()
            .filter(|field| !field.is_flag())
            .map(Field::syntax)
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn example(&self) -> String {
        self.fields
            .iter()
            .filter_map(Field::example)
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn parameters(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter(|field| field.is_flag())
    }
}

fn dispatch<'a, C: Command>(cx: &'a mut Cx, stream: &'a mut Stream) -> Boxed<'a, Result<Response>> {
    Box::pin(async move {
        let parsed = C::parse(cx, stream).await?;

        cx.trace("parse");
        cx.remember(C::META.name, parsed.snapshot());

        parsed.run(cx).await
    })
}

fn rehearse<'a, C: Command>(
    cx: &'a mut Cx,
    stream: &'a mut Stream,
) -> Boxed<'a, Result<serde_json::Value>> {
    Box::pin(async move { Ok(C::parse(cx, stream).await?.snapshot()) })
}

#[derive(Default)]
pub struct Registry {
    entries: Vec<Entry>,
    index: HashMap<&'static str, usize>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<C: Command>(&mut self) {
        let entry = Entry::of::<C>();
        let position = self.entries.len();

        self.entries.push(entry);

        for name in once(&C::META.name).chain(C::META.aliases) {
            assert!(
                self.index.insert(name, position).is_none(),
                "two commands answer to {name}"
            );
        }
    }

    pub fn find(&self, name: &str) -> Option<&Entry> {
        if let Some(position) = self.index.get(name) {
            return self.entries.get(*position);
        }

        let lowered = name.to_lowercase();

        self.index
            .get(lowered.as_str())
            .and_then(|position| self.entries.get(*position))
    }

    pub fn all(&self) -> &[Entry] {
        &self.entries
    }

    pub fn in_category(&self, category: Category) -> impl Iterator<Item = &Entry> {
        self.entries
            .iter()
            .filter(move |entry| entry.meta.category == category)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[macro_export]
macro_rules! register {
    ($registry:expr, $($command:ty),+ $(,)?) => {
        $($registry.add::<$command>();)+
    };
}
