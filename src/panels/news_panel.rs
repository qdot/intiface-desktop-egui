use crate::core::{AppCore, news_file_path, save_config_file};
use super::easy_mark::easy_mark;
use eframe::egui;

pub struct NewsPanel {
  news_str: String
}

fn load_news_file() -> String {
    let news_file = news_file_path();
    if !news_file.exists() {
      "News file not available. Please run update.".to_owned()
    } else {
      std::fs::read_to_string(news_file).unwrap()
    }
}

impl Default for NewsPanel {
  fn default() -> Self {
    Self {
      news_str: load_news_file()
    }
  }
}

impl NewsPanel {

  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    if core.config.unread_news() {
      *core.config.unread_news_mut() = false;
      save_config_file(&serde_json::to_string(&core.config).unwrap()).unwrap();
      self.news_str = load_news_file();
    }
    easy_mark(ui, &self.news_str);
  }
}
