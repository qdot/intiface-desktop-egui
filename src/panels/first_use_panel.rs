use crate::core::{device_config_file_path, engine_file_path, save_config_file, AppCore};
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct FirstUsePanel {}

#[derive(Debug, Clone, Copy)]
enum FirstUseState {
  Intro,
  DownloadCheck,
  DeviceWizard,
  AllowCrashReporting,
  Finish,
}

impl FirstUsePanel {
  fn intro(&self, ui: &mut egui::Ui) {
    let mut clicked = false;
    ui.with_layout(
      egui::Layout::centered_and_justified(egui::Direction::TopDown),
      |ui| {
        ui.set_max_height(ui.available_height() * 0.80);
        ui.set_max_width(ui.available_width() * 0.80);
        ui.vertical(|ui| {
          ui.label("Hello and welcome to the Intiface Desktop first time experience.");
          if ui.button("Continue").clicked() {
            clicked = true;
          }
        });
      },
    );
    if clicked {
      let id = ui.make_persistent_id("FirstUsePanel::FirstUseState");
      ui.memory().data.remove::<FirstUseState>(id);
      ui.memory()
        .data
        .insert_temp(id, FirstUseState::DownloadCheck);
    }
  }

  fn download_check(&self, core: &mut AppCore, ui: &mut egui::Ui) {
    // Check for engine existence
    let engine_exists = engine_file_path().exists();

    // Check for device file existence
    let device_file_exists = device_config_file_path().exists();

    // Check for updates

    // If engine or device file don't exist, or if updates are available, prompt for download
    let mut clicked = false;
    ui.with_layout(
      egui::Layout::centered_and_justified(egui::Direction::TopDown),
      |ui| {
        ui.set_max_height(ui.available_height() * 0.80);
        ui.set_max_width(ui.available_width() * 0.80);
        ui.vertical(|ui| {

          if !core.update_manager.is_updating() {
            ui.label("Now doing download check");
            if engine_exists {
              ui.label("- Engine Exists");
            } else {
              ui.label("- Engine Download Needed");
            }
            if device_file_exists {
              ui.label("- Device File Exists");
            } else {
              ui.label("- Device File Download Needed");
            }
  
            if engine_exists {
              ui.label(RichText::new("You should be able to run Intiface Desktop without updates.").strong());
            } else {
              ui.label(RichText::new("You will need to get updates to run Intiface Desktop, otherwise the program will not work.").strong());
            }
            /*
            if ui.button("Get Updates").clicked() {
              core.update_manager.check_for_and_get_updates();
            }
            */
  
            if ui.button("Continue").clicked() {
              clicked = true;
            }
          } else {
            ui.label("Running updates, please wait a moment.");
          }
        });
      },
    );
    if clicked {
      let id = ui.make_persistent_id("FirstUsePanel::FirstUseState");
      ui.memory().data.remove::<FirstUseState>(id);
      ui.memory()
        .data
        .insert_temp(id, FirstUseState::DeviceWizard);
    }
  }

  fn device_wizard(&self, core: &mut AppCore, ui: &mut egui::Ui) {
    let mut clicked = false;
    ui.with_layout(
      egui::Layout::centered_and_justified(egui::Direction::TopDown),
      |ui| {
        ui.set_max_height(ui.available_height() * 0.80);
        ui.set_max_width(ui.available_width() * 0.80);
        ui.vertical(|ui| {
          ui.label("Now doing device wizard");
          if ui.button("Continue").clicked() {
            clicked = true;
          }
        });
      },
    );
    if clicked {
      let id = ui.make_persistent_id("FirstUsePanel::FirstUseState");
      ui.memory().data.remove::<FirstUseState>(id);
      ui.memory()
        .data
        .insert_temp(id, FirstUseState::AllowCrashReporting);
    }
  }

  fn allow_crash_reporting(&self, core: &mut AppCore, ui: &mut egui::Ui) {
    let mut clicked = false;
    ui.with_layout(
      egui::Layout::centered_and_justified(egui::Direction::TopDown),
      |ui| {
        ui.set_max_height(ui.available_height() * 0.80);
        ui.set_max_width(ui.available_width() * 0.80);
        ui.vertical(|ui| {
          ui.label("Now asking about crash reporting.");
          if ui.button("Continue").clicked() {
            clicked = true;
          }
        });
      },
    );
    if clicked {
      let id = ui.make_persistent_id("FirstUsePanel::FirstUseState");
      ui.memory().data.remove::<FirstUseState>(id);
      ui.memory().data.insert_temp(id, FirstUseState::Finish);
    }
  }

  fn finish(&self, core: &mut AppCore, ui: &mut egui::Ui) {
    let mut clicked = false;
    ui.with_layout(
      egui::Layout::centered_and_justified(egui::Direction::TopDown),
      |ui| {
        ui.set_max_height(ui.available_height() * 0.80);
        ui.set_max_width(ui.available_width() * 0.80);
        ui.vertical(|ui| {
          ui.label("All done, enjoy Intiface Desktop!");
          if ui.button("Continue").clicked() {
            clicked = true;
          }
        });
      },
    );
    if clicked {
      let id = ui.make_persistent_id("FirstUsePanel::FirstUseState");
      ui.memory().data.remove::<FirstUseState>(id);
      *core.config.has_run_first_use_mut() = true;
      save_config_file(&serde_json::to_string(&core.config).unwrap()).unwrap();
    }
  }

  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    let id = ui.make_persistent_id("FirstUsePanel::FirstUseState");
    let panel_state = ui
      .memory()
      .data
      .get_temp_mut_or(id, FirstUseState::Intro)
      .clone();
    match panel_state {
      FirstUseState::Intro => self.intro(ui),
      FirstUseState::DownloadCheck => self.download_check(core, ui),
      FirstUseState::DeviceWizard => self.device_wizard(core, ui),
      FirstUseState::AllowCrashReporting => self.allow_crash_reporting(core, ui),
      FirstUseState::Finish => self.finish(core, ui),
    }
  }
}
