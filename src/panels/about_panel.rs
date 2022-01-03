use crate::core::AppCore;
use eframe::egui;

#[derive(Default)]
pub struct AboutPanel {}

impl AboutPanel {
  pub fn update(&mut self, _core: &mut AppCore, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
      ui.heading("Intiface Desktop");
      ui.label(format!("egui Edition -  Version {} (git {})", env!("VERGEN_GIT_SEMVER"), env!("VERGEN_GIT_SHA_SHORT")));
      ui.label(format!("Built: {}", env!("VERGEN_BUILD_TIMESTAMP")));
      ui.label("Copyright Nonpolynomial, 2017-2021");
      ui.label("Intiface is a registered trademark for Nonpolynomial");
      ui.separator();
      ui.label("For questions or issues relating to Intiface Desktop, please try:");
      ui.hyperlink_to(
        "📖 Read the Intiface Desktop Manual",
        "https://discord.buttplug.io",
      );
      ui.hyperlink_to(
        " Tweet at or DM the buttplug.io Twitter Account",
        "https://twitter.com/buttplugio",
      );
      ui.hyperlink_to(
        "🎮 Join buttplug.io Discord Server",
        "https://discord.buttplug.io",
      );
      ui.hyperlink_to(
        " File an issue on the Intiface Desktop repo",
        "https://github.com/intiface/intiface-desktop",
      );
    });
  }
}
