/// Clock used for all timestamps.
pub type Clock = chrono::Utc;


/// Type for all timestamps.
pub type Timestamp = chrono::DateTime::<Clock>;


lazy_static::lazy_static!(

pub(crate) static ref JANUARY_1970: Timestamp = Timestamp::from_timestamp(0, 0)
    .expect("Zero is a valid timestamp");


pub(crate) static ref FIRST_AFTER_JANUARY_1970: Timestamp = Timestamp::from_timestamp(1, 0)
    .expect("One second after January 1970 is a valid timestamp");

);
