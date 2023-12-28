use std::sync::mpsc::{self, Receiver, Sender};

use ehttp::Response;
use poll_promise::Promise;

use super::asset_loader::{
    exercise_config_collection::ExerciseConfigCollection, perhabs_config::PerhabsConfig,
};

/// AppData is loaded when launching Perhabs. Individual modules/windows get app-wide
/// data through a reference to this struct.
pub struct AppData {
    pub debug: bool,
    pub config: Option<PerhabsConfig>,
    pub config_promise: Option<Promise<ehttp::Result<Response>>>,
    pub excconfig: Option<ExerciseConfigCollection>,
    pub excconfig_promise: Option<Promise<ehttp::Result<Response>>>,
    pub debug_messages: Vec<String>,
    pub error_tx: Sender<String>,
    pub error_rx: Receiver<String>,
}

impl Default for AppData {
    fn default() -> Self {
        let (error_tx, error_rx) = mpsc::channel();
        Self {
            debug: false,
            config: None,
            config_promise: None,
            excconfig: None,
            excconfig_promise: None,
            debug_messages: vec![],
            error_tx,
            error_rx,
        }
    }
}
