use pueue_lib::network::message::GroupResponseMessage;

use super::{helper::*, OutputStyle};

pub fn print_groups(message: GroupResponseMessage, style: &OutputStyle) {
    let mut text = String::new();
    let mut group_iter = message.groups.iter().peekable();
    while let Some((name, group)) = group_iter.next() {
        let styled = get_group_headline(name, group, style);

        text.push_str(&styled);
        if group_iter.peek().is_some() {
            text.push('\n');
        }
    }
    println!("{}", text);
}
