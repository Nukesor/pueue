use std::{collections::HashMap, env::vars, path::PathBuf};

use chrono::Local;
use criterion::{Criterion, criterion_group, criterion_main};
use pueue::daemon::internal_state::state::InternalState;
use pueue_lib::{Settings, Task, state::PUEUE_DEFAULT_GROUP};

/// Create a large state file with a few hundred tasks.
/// Save it to disk in uncompressed state
fn save_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut settings = Settings::default();
    settings.shared.pueue_directory = Some(dir.path().to_owned());
    settings.daemon.compress_state_file = false;

    let mut state = InternalState::new();

    for _ in 0..400 {
        let task = Task::new(
            "ls".into(),
            PathBuf::from("/tmp"),
            HashMap::from_iter(vars()),
            PUEUE_DEFAULT_GROUP.to_owned(),
            pueue_lib::TaskStatus::Queued {
                enqueued_at: Local::now(),
            },
            Vec::new(),
            0,
            None,
        );

        state.add_task(task);
    }

    state.save(&settings).unwrap();
}

pub fn state(crit: &mut Criterion) {
    crit.bench_function("Save uncompressed state", |b| b.iter(save_state));
}

criterion_group!(benches, state);
criterion_main!(benches);
