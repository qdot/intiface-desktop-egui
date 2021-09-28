// Taken from tracing-egui by CAD97

use std::sync::Arc;

use super::archive::*;
use tracing::span;
use tracing_subscriber::{layer, registry::LookupSpan, Layer};

pub struct EguiLayer {
    _priv: (),
}

impl EguiLayer {
    pub fn new() -> Self {
        #[cfg(feature = "smartstring")]
        smartstring::validate();
        EguiLayer { _priv: () }
    }
}

impl Default for EguiLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for EguiLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn new_span(&self, fields: &span::Attributes<'_>, id: &span::Id, ctx: layer::Context<'_, S>) {
        let meta = ctx.metadata(id);
        let span = ctx.span(id).expect("ctx span");
        let mut ext = span.extensions_mut();

        if ext.get_mut::<Arc<LogSpan>>().is_none() {
            let mut log = LogSpan {
                meta,
                fields: Default::default(),
                parent: span
                    .parent()
                    .and_then(|span| span.extensions().get::<Arc<LogSpan>>().map(Arc::clone)),
            };
            if log.add_fields(fields).is_ok() {
                ext.insert(Arc::new(log));
            }
        }
    }

    fn on_record(&self, id: &span::Id, fields: &span::Record<'_>, ctx: layer::Context<'_, S>) {
        let meta = ctx.metadata(id);
        let span = ctx.span(id).expect("ctx span");
        let mut ext = span.extensions_mut();

        match ext.get_mut::<Arc<LogSpan>>() {
            Some(log) => {
                let log = Arc::make_mut(log);
                let _ = log.add_fields(fields);
            }
            None => {
                let mut log = LogSpan {
                    meta,
                    fields: Default::default(),
                    parent: span
                        .parent()
                        .and_then(|span| span.extensions().get::<Arc<LogSpan>>().map(Arc::clone)),
                };
                if log.add_fields(fields).is_ok() {
                    ext.insert(Arc::new(log));
                }
            }
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: layer::Context<'_, S>) {
        let meta = event.metadata();
        let span = ctx.event_span(event);

        let mut log = LogEvent {
            meta,
            timestamp: chrono::Local::now().naive_local(),
            fields: Default::default(),
            span: span.and_then(|span| span.extensions().get::<Arc<LogSpan>>().map(Arc::clone)),
        };
        let _ = log.add_fields(event);
        LOG_ENTRIES.lock().push(log);
    }
}