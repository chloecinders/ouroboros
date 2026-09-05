use std::sync::Arc;
use std::vec::IntoIter;

use sha2::{Digest, Sha256};
use tokio::task::JoinSet;

use crate::domain::Snowflake;
use crate::features::automod::rule::Source;
use crate::features::automod::{cache, origin, readings, sources, store};
use crate::platform::discord::dispatch::MessageCx;
use crate::platform::observe::report::Origin;
use crate::platform::text::fuzzy::Haystack;
use crate::platform::{ocr, text};

pub struct Images {
    lanes: JoinSet<Option<String>>,
    waiting: IntoIter<sources::Readable>,
    app: Arc<crate::app::App>,
    enabled: cache::Enabled,
    rules: String,
    where_: Origin,
    message: Snowflake,
}

impl Images {
    pub fn open(cx: &MessageCx, enabled: &cache::Enabled) -> Option<Self> {
        let rules = cache::image_hash(enabled)?;
        let waiting = sources::readable(&cx.msg);

        if waiting.is_empty() {
            return None;
        }

        Some(Self {
            lanes: JoinSet::new(),
            waiting: waiting.into_iter(),
            app: Arc::clone(&cx.app),
            enabled: cache::Enabled::clone(enabled),
            rules,
            where_: origin(cx),
            message: cx.msg.id.get(),
        })
    }

    fn fill(&mut self) {
        while self.lanes.len() < 4 {
            let Some(target) = self.waiting.next() else {
                break;
            };

            let app = Arc::clone(&self.app);
            let enabled = cache::Enabled::clone(&self.enabled);
            let rules = self.rules.clone();
            let (where_, message) = (self.where_, self.message);

            self.lanes.spawn(async move {
                let bytes = target.fetch(&app.http).await.ok()?;
                let image = text::hex(&Sha256::digest(&bytes));

                if let Ok(Some(false)) = store::evaluated(&app.pool, &image, &rules).await {
                    return None;
                }

                let text = ocr::read(&bytes).await.map(|reading| reading.text)?;

                app.readings.remember(
                    message,
                    readings::Reading {
                        fingerprint: image.clone(),
                        text: text.clone(),
                    },
                );

                let read = Haystack::new(&text);
                let matched = enabled
                    .iter()
                    .filter(|rule| rule.body.has_source(Source::Image))
                    .any(|rule| rule.body.matches.iter().any(|matcher| matcher.test(&read)));

                if let Err(failure) = store::remember(&app.pool, &image, &rules, matched).await {
                    app.reporter.record(&failure, where_);
                }

                Some(text)
            });
        }
    }

    pub async fn next(&mut self) -> Option<String> {
        loop {
            self.fill();

            match self.lanes.join_next().await? {
                Ok(Some(text)) => return Some(text),
                _ => continue,
            }
        }
    }
}
