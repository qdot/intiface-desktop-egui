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

#[derive(Debug, Clone, Copy)]
enum DownloadState {
  Starting,
  Downloading,
  WaitForDownloads,
  Done
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
          ui.label("Hello and welcome to Intiface Desktop! Before we get started controlling toys, we need to configure a few things.");
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
    let state_id = ui.make_persistent_id("FirstUsePanel::DownloadState");
    let state = ui.memory().data.get_temp_mut_or(state_id, DownloadState::Starting).clone();
    
    let mut new_state = state;
    match state {
      DownloadState::Starting => {
        ui.with_layout(
          egui::Layout::centered_and_justified(egui::Direction::TopDown),
          |ui| {
            ui.set_max_height(ui.available_height() * 0.80);
            ui.set_max_width(ui.available_width() * 0.80);
            ui.vertical(|ui| {    
              ui.label("Now doing download check");
              if engine_file_path().exists() {
                ui.label("- Engine Exists");
              } else {
                ui.label("- Engine Download Needed");
              }
              if device_config_file_path().exists() {
                ui.label("- Device File Exists");
              } else {
                ui.label("- Device File Download Needed");
              }
    
              if engine_file_path().exists() {
                ui.label(RichText::new("You should be able to run Intiface Desktop without updates.").strong());
              } else {
                ui.label(RichText::new("You will need to get updates to run Intiface Desktop, otherwise the program will not work.").strong());
              }
  
              if ui.button("Download Updates (If Available, Optional)").clicked() {
                new_state = DownloadState::Downloading;
              }
    
              if ui.button("Continue").clicked() {
                new_state = DownloadState::Done;
              }
            });
          });
      },
      DownloadState::Downloading => {
        core.update_manager.get_updates();
        new_state = DownloadState::WaitForDownloads;
      },
      DownloadState::WaitForDownloads => {
        if core.update_manager.is_updating() {
          ui.label("Please waiting, downloading updates...");
        } else {
          ui.vertical(|ui|{
            ui.label("Downloads finished!");
            if ui.button("Continue").clicked() {
              new_state = DownloadState::Done;
            }
          });
        }
      },
      DownloadState::Done => {
        let id = ui.make_persistent_id("FirstUsePanel::FirstUseState");
        ui.memory().data.remove::<FirstUseState>(id);
        ui.memory()
          .data
          .insert_temp(id, FirstUseState::DeviceWizard);
      }
    }
    ui.memory().data.remove::<DownloadState>(state_id);
    ui.memory().data.insert_temp(state_id, new_state);
  }

  fn device_wizard(&self, core: &mut AppCore, ui: &mut egui::Ui) {
    let mut clicked = false;
    ui.with_layout(
      egui::Layout::centered_and_justified(egui::Direction::TopDown),
      |ui| {
        ui.set_max_height(ui.available_height() * 0.80);
        ui.set_max_width(ui.available_width() * 0.80);
        ui.vertical(|ui| {
          ui.label("The following connection types are available in Buttplug. Please uncheck any you don't plan on using. These can be configured in the Settings panel later if changes need to be made.");
          ui.label(RichText::new("For Lovense Users:").strong());
          ui.label("Only choose one of Bluetooth LE, Lovense Dongle, or Lovense Connect. Having multiple of these on with a Lovense toy can cause conflicts. Bluetooth LE is the recommended connection method if available.");
          super::settings_panel::render_device_connection_types(core, ui);
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
          ui.label("Note that, for the beta, crash reporting is turned on by default. This allows us to find and fix bugs before the full public release. In the full public release, crash logging will be off by default. If you object to crash logging during the beta period, you can disable it in settings.");
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
