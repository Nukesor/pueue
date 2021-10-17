use crossbeam_channel::Sender;

use pueue_lib::network::message::*;
use pueue_lib::settings::PUEUE_DEFAULT_GROUP;
use pueue_lib::state::SharedState;

use crate::network::message_handler::ok_or_failure_message;
use crate::network::response_helper::ensure_group_exists;
use crate::ok_or_return_failure_message;

/// Invoked on `pueue groups`.
/// Manage groups.
/// - Show groups
/// - Add group
/// - Remove group
pub fn group(message: GroupMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let state = state.lock().unwrap();

    match message {
        GroupMessage::List => {
            // Return information about all groups to the client.
            Message::GroupResponse(GroupResponseMessage {
                groups: state.groups.clone(),
                settings: state.settings.daemon.groups.clone(),
            })
        }
        GroupMessage::Add { name, parallel_tasks } => {
            if state.groups.contains_key(&name) {
                return create_failure_message(format!("Group \"{}\" already exists", name));
            }

            // Propagate the message to the TaskHandler, which is responsible for actually
            // manipulating our internal data
            let result = sender.send(Message::Group(GroupMessage::Add{ name: name.clone(), parallel_tasks }));
            ok_or_return_failure_message!(result);

            create_success_message(format!("Group \"{}\" is being created", name))
        }
        GroupMessage::Remove(group) => {
            if let Err(message) = ensure_group_exists(&state, &group) {
                return message;
            }

            if group == PUEUE_DEFAULT_GROUP {
                return create_failure_message("You cannot delete the default group".to_string());
            }

            // Make sure there are no tasks in that group.
            if state.tasks.iter().any(|(_, task)| task.group == group) {
                return create_failure_message(
                    "You cannot remove a group, if there're still tasks in it.".to_string(),
                );
            }

            // Propagate the message to the TaskHandler, which is responsible for actually
            // manipulating our internal data
            let result = sender.send(Message::Group(GroupMessage::Remove(group.clone())));
            ok_or_return_failure_message!(result);

            create_success_message(format!("Group \"{}\" removed", group))
        }
    }
}
