use failure::{Error, Fail};

#[derive(Debug, Fail)]
pub enum DaemonError {
    #[fail(display = "Following entries are not found: {:?}", indices)]
    EntriesNotFound { indices: Vec<usize> },
}
