use std::collections::BTreeMap;

use anyhow::anyhow;
use log::{error, info};

use pueue_lib::network::message::GroupMessage;

use crate::ok_or_shutdown;
use crate::state_helper::save_state;
use crate::task_handler::{Shutdown, TaskHandler};

impl TaskHandler {
    /// Handle the addition and the removal of groups.
    ///
    /// This is done in the TaskHandler, as we also have to create/remove worker pools.
    /// I.e. we have to touch three things:
    /// - state.groups
    /// - state.config.daemon.groups
    /// - self.children
    pub fn handle_group_message(&mut self, message: GroupMessage) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();

        match message {
            GroupMessage::List => {}
            GroupMessage::Add {
                name,
                parallel_tasks,
            } => {
                if state.groups.contains_key(&name) {
                    error!("Group \"{}\" already exists", name);
                    return;
                }
                let mut group = state.create_group(&name);
                if let Some(parallel_tasks) = parallel_tasks {
                    group.parallel_tasks = parallel_tasks;
                }
                info!("New group \"{}\" has been created", &name);

                // Create the worker pool.
                self.children.0.insert(name, BTreeMap::new());

                // Persist the state.
                ok_or_shutdown!(self, save_state(&state));
            }
            GroupMessage::Remove(group) => {
                if !state.groups.contains_key(&group) {
                    error!("Group \"{}\" to be remove doesn't exists", group);
                    return;
                }

                // Make sure there are no tasks in that group.
                if state.tasks.iter().any(|(_, task)| task.group == group) {
                    error!(
                        "Tried to remove group \"{}\", while it still contained tasks.",
                        group
                    );
                    return;
                }

                if let Err(error) = state.remove_group(&group) {
                    error!("Error while removing group: \"{}\"", error);
                    return;
                }

                // Make sure the worker pool exists and is empty.
                // There shouldn't be any children, if there are no tasks in this group.
                // Those are critical errors, as they indicate desynchronization inside our
                // internal datastructures, which is really bad.
                if let Some(pool) = self.children.0.get(&group) {
                    if !pool.is_empty() {
                        anyhow!("Encountered a non-empty worker pool, while removing a group. This is a critical error. Please report this bug.");
                        self.initiate_shutdown(Shutdown::Emergency);
                        return;
                    }
                } else {
                    anyhow!("Encountered an group without an worker pool, while removing a group. This is a critical error. Please report this bug.");
                    self.initiate_shutdown(Shutdown::Emergency);
                    return;
                }
                // Actually remove the worker pool.
                self.children.0.remove(&group);

                // Persist the state.
                ok_or_shutdown!(self, save_state(&state));

                info!("Group \"{}\" has been removed", &group);
            }
        }
    }
}
