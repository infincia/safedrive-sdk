use std::str;

/// internal imports

use CANCEL_LIST;

pub fn cancel_sync_task(name: &str) {
    let mut cl = CANCEL_LIST.write();
    cl.push(name.to_owned());
}

pub fn is_sync_task_cancelled(name: String) -> bool {
    let mut cl = CANCEL_LIST.write();
    let mut cancelled = false;

    { if cl.contains(&name) { cancelled = true; } }

    { if cancelled { cl.retain(|&ref x| x != &name); }}

    cancelled
}