use chrono::{DateTime, Local, TimeDelta};

use pueue_lib::settings::Settings;

// If the enqueue at time is today, only show the time. Otherwise, include the date.
pub fn format_datetime(settings: &Settings, enqueue_at: &DateTime<Local>) -> String {
    let format_string = if enqueue_at.date_naive() == Local::now().date_naive() {
        &settings.client.get_status_time_format()
    } else {
        &settings.client.get_status_datetime_format()
    };
    enqueue_at.format(format_string).to_string()
}

pub fn humanize_duration(duration: TimeDelta) -> String {
    let days = duration.num_days();
    let leftover = duration - TimeDelta::days(days);

    let hours = leftover.num_hours();
    let leftover = leftover - TimeDelta::hours(hours);

    let minutes = leftover.num_minutes();
    let leftover = leftover - TimeDelta::minutes(minutes);

    let seconds = leftover.num_seconds();
    let leftover = leftover - TimeDelta::seconds(seconds);

    let millis = leftover.num_milliseconds();

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
    if days == 0 && hours == 0 && minutes == 0 && seconds < 3 && millis > 0 {
        parts.push(format!("{millis}ms"));
    }

    parts.join(" ")
}

#[cfg(test)]
mod test {
    use super::humanize_duration;
    use chrono::TimeDelta;

    macro_rules! assert_eq_seconds {
        ($seconds:expr, $str: expr) => {
            assert_eq!(
                humanize_duration(TimeDelta::new($seconds, 0).unwrap()),
                $str.to_owned()
            );
        };
    }

    macro_rules! assert_eq_millis {
        ($millis:expr, $str: expr) => {
            assert_eq!(
                humanize_duration(TimeDelta::try_milliseconds($millis).unwrap()),
                $str.to_owned()
            );
        };
    }

    #[test]
    fn duration() {
        assert_eq_millis!(500, "500ms");
        assert_eq_millis!(1500, "1sec 500ms");
        assert_eq_millis!(2500, "2sec 500ms");
        assert_eq_millis!(3500, "3sec");
        assert_eq_millis!(1000 *60 + 500, "1min");

        assert_eq_seconds!(1, "1sec");
        assert_eq_seconds!(90, "1min 30sec");
        assert_eq_seconds!(1 * 3600, "1hr");
        assert_eq_seconds!(1 * 3600 * 24, "1day");
        assert_eq_seconds!((1.5 * 3600. * 24.) as i64, "1day 12hr");
        assert_eq_seconds!(2 * 3600 * 24, "2day");
    }
}
