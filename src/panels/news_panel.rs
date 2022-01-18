use crate::core::{AppCore, news_file_path};
use super::easy_mark::easy_mark;
use eframe::egui;

pub struct NewsPanel {
  news_str: String
}

impl Default for NewsPanel {
  fn default() -> Self {
    let news_file = news_file_path();
    let news_str = if !news_file.exists() {
      "News file not available. Please run update.".to_owned()
    } else {
      std::fs::read_to_string(news_file).unwrap()
    };
    Self {
      news_str
    }
  }
}

impl NewsPanel {
  pub fn update(&mut self, _core: &mut AppCore, ui: &mut egui::Ui) {
    easy_mark(ui, &self.news_str);
  }
}
