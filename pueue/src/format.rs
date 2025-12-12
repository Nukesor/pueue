use chrono::{DateTime, Local, TimeDelta};

use pueue_lib::settings::Settings;

// If the enqueue at time is today, only show the time. Otherwise, include the date.
pub fn format_datetime(settings: &Settings, enqueue_at: &DateTime<Local>) -> String {
    let format_string = if enqueue_at.date_naive() == Local::now().date_naive() {
        &settings.client.status_time_format
    } else {
        &settings.client.status_datetime_format
    };
    enqueue_at.format(format_string).to_string()
}

pub fn humanize_duration(d: TimeDelta) -> String {
    let mut millis = d.num_milliseconds();

    let days = millis / 86_400_000;
    millis %= 86_400_000;

    let hours = millis / 3_600_000;
    millis %= 3_600_000;

    let minutes = millis / 60_000;
    millis %= 60_000;

    let seconds = millis / 1000;
    millis %= 1000;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}day"));
    }
    if hours > 0 {
        parts.push(format!("{hours}hr"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}min"));
    }
    if seconds > 0 {
        parts.push(format!("{seconds}sec"));
    }
    if days == 0 && hours == 0 && minutes == 0 && seconds < 3 {
        parts.push(format!("{millis}ms"));
    }

    parts.join(" ")
}
