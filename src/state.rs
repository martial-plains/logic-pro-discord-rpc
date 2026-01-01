use crate::discord::DiscordIpcClientWrapper;
use crate::logic::{get_logic_project_name, is_logic_pro_running};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::{
    sync::{Mutex, OnceLock},
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Debug)]
pub struct AppState {
    client: Mutex<DiscordIpcClientWrapper>,
    running: AtomicBool,
    logic_thread: OnceLock<JoinHandle<()>>,
    last_state: Mutex<Option<String>>,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    pub fn new() -> Option<Arc<Self>> {
        let client = DiscordIpcClientWrapper::new()?; // returns Option<Self>

        let app = Arc::new(Self {
            running: AtomicBool::new(true),
            client: Mutex::new(client),
            logic_thread: OnceLock::new(),
            last_state: Mutex::new(None),
        });

        app.start_logic_thread();

        Some(app)
    }

    fn start_logic_thread(self: &Arc<Self>) {
        let app_clone = Arc::clone(self);
        self.logic_thread.get_or_init(|| {
            thread::spawn(move || {
                app_clone.logic_pro_loop();
            })
        });
    }

    fn logic_pro_loop(&self) {
        let mut was_running = false;

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            let running = is_logic_pro_running();

            if !running {
                if was_running {
                    self.with_client(|c| c.clear_activity());
                }
                was_running = false;
                thread::sleep(Duration::from_secs(1));
                continue;
            }

            was_running = true;

            let state_string = match get_logic_project_name() {
                Some(project) => format!("Working on {}", project),
                None => "Browsing projects".to_string(),
            };

            {
                let mut last = self.last_state.lock().unwrap();
                if last.as_ref() != Some(&state_string) {
                    *last = Some(state_string.clone());
                    self.with_client(|c| c.update_state(&state_string));
                }
            }

            thread::sleep(Duration::from_secs(1));
        }

        self.with_client(|c| c.clear_activity());
    }

    fn with_client<F>(&self, f: F)
    where
        F: FnOnce(&mut DiscordIpcClientWrapper),
    {
        if let Ok(mut client) = self.client.lock() {
            f(&mut client);
        }
    }

    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

static APP: OnceLock<Arc<AppState>> = OnceLock::new();

pub fn start_idle() {
    if APP.get().is_some() {
        return; // Already running
    }

    let app = AppState::new().expect("Failed to initialize AppState");
    APP.set(Arc::clone(&app)).unwrap();
}

pub fn stop() {
    if let Some(app) = APP.get() {
        app.stop();
    }
}
