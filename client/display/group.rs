use pueue::network::message::GroupResponseMessage;

use super::helper::*;

pub fn print_groups(message: GroupResponseMessage) {
    let mut text = String::new();
    let mut group_iter = message.groups.iter().peekable();
    while let Some((name, status)) = group_iter.next() {
        let parallel = *message.settings.get(name).unwrap();
        let styled = get_group_headline(name, &status, parallel);

        text.push_str(&styled);
        if group_iter.peek().is_some() {
            text.push('\n');
        }
    }
    println!("{}", text);
}
