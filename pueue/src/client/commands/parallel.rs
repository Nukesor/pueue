use pueue_lib::{client::Client, network::message::*};

use super::{group_or_default, handle_response};
use crate::{client::style::OutputStyle, internal_prelude::*};

/// Set the parallelization settings for a group or show the current group settings.
pub async fn parallel(
    client: &mut Client,
    style: &OutputStyle,
    parallel_tasks: Option<usize>,
    group: Option<String>,
) -> Result<()> {
    let request: Request = match parallel_tasks {
        Some(parallel_tasks) => {
            let group = group_or_default(&group);
            ParallelMessage {
                parallel_tasks,
                group,
            }
            .into()
        }
        None => GroupMessage::List.into(),
    };

    client.send_request(request).await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
