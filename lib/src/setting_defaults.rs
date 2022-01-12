/// The `Default` impl for `bool` is `false`.
/// This function covers the `true` case.
pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_host() -> String {
    "127.0.0.1".to_string()
}

pub(crate) fn default_port() -> String {
    "6924".to_string()
}

pub(crate) fn default_status_time_format() -> String {
    "%H:%M:%S".to_string()
}

pub(crate) fn default_status_datetime_format() -> String {
    "%Y-%m-%d\n%H:%M:%S".to_string()
}

pub(crate) fn default_callback_log_lines() -> usize {
    10
}
