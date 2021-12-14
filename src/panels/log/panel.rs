// Taken from tracing-egui by CAD97

use super::archive::LOG_ENTRIES;
use eframe::egui::{self, Color32, RichText};
use tracing::{level_filters::STATIC_MAX_LEVEL, Level};

#[derive(Debug, Default)]
pub struct LogPanel;

#[derive(Debug, Clone)]
struct LogPanelState {
  trace: bool,
  debug: bool,
  info: bool,
  warn: bool,
  error: bool,
}

impl Default for LogPanelState {
  fn default() -> Self {
    Self {
      trace: true,
      debug: true,
      info: true,
      warn: true,
      error: true,
    }
  }
}

impl LogPanel {
  pub fn update(self, ui: &mut egui::Ui) {
    let id = ui.make_persistent_id("tracing-egui::LogPanel");
    let mut state = ui
      .memory()
      .data
      .get_temp_mut_or_default::<LogPanelState>(id)
      .clone();

    egui::TopBottomPanel::top("Log Levels").show(ui.ctx(), |ui| {
      ui.horizontal(|ui| {
        if Level::TRACE < STATIC_MAX_LEVEL {
          ui.checkbox(&mut state.trace, "trace");
        }
        if Level::DEBUG < STATIC_MAX_LEVEL {
          ui.checkbox(&mut state.debug, "debug");
        }
        if Level::INFO < STATIC_MAX_LEVEL {
          ui.checkbox(&mut state.info, "info");
        }
        if Level::WARN < STATIC_MAX_LEVEL {
          ui.checkbox(&mut state.warn, "warn");
        }
        if Level::ERROR < STATIC_MAX_LEVEL {
          ui.checkbox(&mut state.error, "error");
        }
      })
    });

    egui::TopBottomPanel::bottom("Log Buttons").show(ui.ctx(), |ui| {
      ui.horizontal(|ui| {
        if ui.button("Send Logs To Sentry").clicked() {}
        if ui.button("Clear Log Display").clicked() {
          let mut log_entries = LOG_ENTRIES.lock();
          log_entries.clear();
        }
      })
    });

    egui::CentralPanel::default().show(ui.ctx(), |ui| {
      egui::ScrollArea::vertical().id_source("log_panel").show(ui, |ui| {
        let log_entries = LOG_ENTRIES.lock();
        for (log_ix, log) in log_entries.iter().enumerate().rev() {
          let filtered_out = match *log.meta.level() {
            Level::TRACE => !state.trace,
            Level::DEBUG => !state.debug,
            Level::INFO => !state.info,
            Level::WARN => !state.warn,
            Level::ERROR => !state.error,
          };
          if filtered_out {
            continue;
          }

          let log_id = id.with(log_ix);
          match log.fields.get("message") {
            Some(message) => egui::CollapsingHeader::new(
              RichText::new(format!(
                "[{}] [{}] {}",
                log.timestamp.format("%H:%M:%S%.3f"),
                log.meta.level(),
                message,
              ))
              .color(match *log.meta.level() {
                Level::DEBUG => Color32::BLUE,
                Level::ERROR => Color32::LIGHT_RED,
                Level::WARN => Color32::YELLOW,
                Level::INFO => Color32::GREEN,
                Level::TRACE => Color32::LIGHT_GRAY,
              }),
            ),
            None => egui::CollapsingHeader::new(format!(
              "[{}] [{}]",
              log.timestamp.format("%H:%M:%S%.3f"),
              log.meta.level(),
            )),
          }
          .id_source(log_id)
          .show(ui, |ui| {
            egui::CollapsingHeader::new(format!("{} {}", log.meta.target(), log.meta.name(),))
              .id_source(log_id.with(0usize))
              .show(ui, |ui| {
                log.show_fields(ui);
              });

            for (span_ix, span) in
              std::iter::successors(log.span.as_deref(), |span| span.parent.as_deref()).enumerate()
            {
              let span_id = log_id.with(span_ix + 1);
              egui::CollapsingHeader::new(format!(
                "{}::{}",
                span.meta.map_or("{unknown}", |meta| meta.target()),
                span.meta.map_or("{unknown}", |meta| meta.name()),
              ))
              .id_source(span_id)
              .show(ui, |ui| {
                span.show_fields(ui);
              });
            }
          });
        }
      })
    });
    ui.memory().data.insert_persisted(id, state);
  }
}
