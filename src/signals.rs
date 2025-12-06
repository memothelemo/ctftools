use log::debug;
use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

static DEFAULT_SIGNALS: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| {
    let arc = Arc::new(AtomicBool::new(true));
    signal_flag::register_conditional_default(SIGTERM, Arc::clone(&arc)).unwrap();
    signal_flag::register_conditional_default(SIGINT, Arc::clone(&arc)).unwrap();
    #[cfg(unix)]
    signal_flag::register_conditional_default(SIGQUIT, Arc::clone(&arc)).unwrap();
    arc
});

pub fn lock_terminate_signals() {
    DEFAULT_SIGNALS.store(true, Ordering::Relaxed);
    debug!("terminate signals locked");
}

pub fn unlock_terminate_signals() {
    DEFAULT_SIGNALS.store(false, Ordering::Relaxed);
    debug!("terminate signals unlocked");
}

pub fn hook() {
    _ = &*DEFAULT_SIGNALS;
    debug!("hooked termination signals");
}
