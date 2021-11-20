// Taken from tracing-egui by CAD97

use super::SmartString;
use hashbrown::hash_map::DefaultHashBuilder;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{fmt::Write, sync::Arc};
use eframe::egui;

#[derive(Clone)]
pub struct LogSpan {
    pub meta: Option<&'static tracing::Metadata<'static>>,
    pub fields: IndexMap<&'static str, SmartString, DefaultHashBuilder>,
    pub parent: Option<Arc<LogSpan>>,
}

pub struct LogEvent {
    pub meta: &'static tracing::Metadata<'static>,
    pub timestamp: chrono::NaiveDateTime,
    pub fields: IndexMap<&'static str, SmartString, DefaultHashBuilder>,
    pub span: Option<Arc<LogSpan>>,
}

pub static LOG_ENTRIES: Lazy<Mutex<Vec<LogEvent>>> = Lazy::new(Default::default);

struct HashMapFieldRecordVisitor<'a> {
    target: &'a mut IndexMap<&'static str, SmartString, DefaultHashBuilder>,
}

impl<'a> HashMapFieldRecordVisitor<'a> {
    fn of(target: &'a mut IndexMap<&'static str, SmartString, DefaultHashBuilder>) -> Self {
        Self { target }
    }
}

impl<'a> tracing::field::Visit for HashMapFieldRecordVisitor<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.target
            .entry(field.name())
            .and_modify(|entry| write!(entry, ", {}", value).unwrap())
            .or_insert_with(|| value.into());
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.target
            .entry(field.name())
            .and_modify(|entry| write!(entry, ", {:?}", value).unwrap())
            .or_insert_with(|| format!("{:?}", value).into());
    }
}

impl LogSpan {
    pub fn add_fields<R>(&mut self, fields: R) -> std::fmt::Result
    where
        R: tracing_subscriber::field::RecordFields,
    {
        fields.record(&mut HashMapFieldRecordVisitor::of(&mut self.fields));
        Ok(())
    }

    pub fn show_fields(&self, ui: &mut egui::Ui) {
        for (field, value) in &self.fields {
            ui.add(egui::Label::new(&format!("{}: {}", field, value)));
        }
    }
}

impl LogEvent {
    pub fn add_fields<R>(&mut self, fields: R) -> std::fmt::Result
    where
        R: tracing_subscriber::field::RecordFields,
    {
        fields.record(&mut HashMapFieldRecordVisitor::of(&mut self.fields));
        Ok(())
    }

    pub fn show_fields(&self, ui: &mut egui::Ui) {
        for (field, value) in &self.fields {
            ui.add(egui::Label::new(format!("{}: {}", field, value)));
        }
    }
}