use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineMessage {
  MessageVersion(u32),
  EngineLog(String),
  EngineStarted,
  EngineError(String),
  EngineStopped,
  ClientConnected(String),
  ClientDisconnected,
  DeviceConnected {
    name: String,
    index: u32,
    address: String,
    display_name: String,
  },
  DeviceDisconnected(u32),
  ClientRejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntifaceMessage {
  Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineLogMessage {
  #[serde(default)]
  timestamp: String,
  #[serde(default)]
  level: String,
  #[serde(default)]
  fields: EngineLogMessageFields,
  #[serde(default)]
  target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineLogMessageFields {
  message: String,
  #[serde(rename = "log.module_path", default)]
  module_path: String,
  #[serde(rename = "log.file", default)]
  file: String,
  #[serde(rename = "log.line", default)]
  line: u32,
}

impl EngineLogMessage {
  pub fn log_event(&self) {
    let span = tracing::span!(tracing::Level::ERROR, "intiface_cli");
    let _enter = span.enter();
    let level = tracing::Level::from_str(&self.level).unwrap();
    match level {
      tracing::Level::ERROR => {
        tracing::error!(
          engine_target = ?self.target,
          engine_file = ?self.fields.file,
          engine_line = self.fields.line,
          engine_module_path = ?self.fields.module_path,
          "{}", self.fields.message
        );
      }
      tracing::Level::WARN => {
        tracing::warn!(
          engine_target = ?self.target,
          engine_file = ?self.fields.file,
          engine_line = self.fields.line,
          engine_module_path = ?self.fields.module_path,
          "{}", self.fields.message
        );
      }
      tracing::Level::INFO => {
        tracing::info!(
          engine_target = ?self.target,
          engine_file = ?self.fields.file,
          engine_line = self.fields.line,
          engine_module_path = ?self.fields.module_path,
          "{}", self.fields.message
        );
      }
      tracing::Level::DEBUG => {
        tracing::debug!(
          engine_target = ?self.target,
          engine_file = ?self.fields.file,
          engine_line = self.fields.line,
          engine_module_path = ?self.fields.module_path,
          "{}", self.fields.message
        );
      }
      tracing::Level::TRACE => {
        tracing::trace!(
          engine_target = ?self.target,
          engine_file = ?self.fields.file,
          engine_line = self.fields.line,
          engine_module_path = ?self.fields.module_path,
          "{}", self.fields.message
        );
      }
    };
  }
}
