use log::{error, info};

use pueue_lib::network::message::GroupMessage;

use crate::ok_or_shutdown;
use crate::state_helper::{save_settings, save_state};
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
            GroupMessage::Add(group) => {
                if state.groups.contains_key(&group) {
                    error!("Group \"{}\" already exists", group);
                    return;
                }
                state.create_group(&group);
                info!("New group \"{}\" has been created", &group);

                // Save the state and the settings file.
                ok_or_shutdown!(self, save_state(&state));
                ok_or_shutdown!(self, save_settings(&state));
            }
            GroupMessage::Remove(group) => {
                if !state.groups.contains_key(&group) {
                    error!("Group \"{}\" to be remove doesn't exists", group);
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

                // Save the state and the settings file.
                ok_or_shutdown!(self, save_state(&state));
                ok_or_shutdown!(self, save_settings(&state));

                info!("Group \"{}\" has been removed", &group);
            }
        }
    }
}
