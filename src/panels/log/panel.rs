// Taken from tracing-egui by CAD97

use crate::core::AppCore;
use super::archive::LOG_ENTRIES;
use eframe::egui::{self, Color32, RichText};
use tracing::{level_filters::STATIC_MAX_LEVEL, Level};

#[derive(Debug, Default)]
pub struct LogPanel {
  state: LogPanelState,
}

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
      trace: false,
      debug: true,
      info: true,
      warn: true,
      error: true,
    }
  }
}

impl LogPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    // Needed for log line container indexing.
    let id = ui.make_persistent_id("tracing-egui::LogPanel");

    // if we've forced the log panel open, wind to the latest error message.
    if core.config.force_open_log() {
      *core.config.force_open_log_mut() = false;
    }

    egui::TopBottomPanel::top("Log Levels").show(ui.ctx(), |ui| {
      ui.horizontal(|ui| {
        if Level::TRACE < STATIC_MAX_LEVEL {
          ui.checkbox(&mut self.state.trace, "trace");
        }
        if Level::DEBUG < STATIC_MAX_LEVEL {
          ui.checkbox(&mut self.state.debug, "debug");
        }
        if Level::INFO < STATIC_MAX_LEVEL {
          ui.checkbox(&mut self.state.info, "info");
        }
        if Level::WARN < STATIC_MAX_LEVEL {
          ui.checkbox(&mut self.state.warn, "warn");
        }
        if Level::ERROR < STATIC_MAX_LEVEL {
          ui.checkbox(&mut self.state.error, "error");
        }
      })
    });

    egui::TopBottomPanel::bottom("Log Buttons").show(ui.ctx(), |ui| {
      ui.horizontal(|ui| {
        if ui.button("Send Logs To Sentry").clicked() {
          sentry::capture_message("User requested to send a log.", sentry::Level::Info);
        }
        if ui.button("Clear Log Display").clicked() {
          let mut log_entries = LOG_ENTRIES.lock();
          log_entries.clear();
        }
      })
    });

    egui::CentralPanel::default().show(ui.ctx(), |ui| {
      egui::ScrollArea::both()
        .id_source("log_panel")
        .show(ui, |ui| {
          ui.set_min_width(ui.available_width());
          let log_entries = LOG_ENTRIES.lock();
          for (log_ix, log) in log_entries.iter().enumerate().rev() {
            let filtered_out = match *log.meta.level() {
              Level::TRACE => !self.state.trace,
              Level::DEBUG => !self.state.debug,
              Level::INFO => !self.state.info,
              Level::WARN => !self.state.warn,
              Level::ERROR => !self.state.error,
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
                std::iter::successors(log.span.as_deref(), |span| span.parent.as_deref())
                  .enumerate()
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
  }
}
