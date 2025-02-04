use chrono::{DateTime, Local};

use crate::settings::Settings;

// If the enqueue at time is today, only show the time. Otherwise, include the date.
pub fn format_datetime(settings: &Settings, enqueue_at: &DateTime<Local>) -> String {
    let format_string = if enqueue_at.date_naive() == Local::now().date_naive() {
        &settings.client.status_time_format
    } else {
        &settings.client.status_datetime_format
    };
    enqueue_at.format(format_string).to_string()
}
