//! Time-related utilities

/// A timebase is a rational number that represents the unit of time for timestamps.
/// It is represented as a fraction num/den.
/// For example, a timebase of 1/1000 represents milliseconds, while a timebase of 1/48000 represents samples in audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBase {
    /// The numerator of the timebase fraction.
    pub num: u32,

    /// The denominator of the timebase fraction.
    pub den: u32,
}

impl TimeBase {
    /// Creates a new TimeBase with the given numerator and denominator.
    pub const fn new(num: u32, den: u32) -> TimeBase {
        TimeBase { num, den }
    }

    /// Converts the timebase to a floating-point number.
    pub fn as_f32(&self) -> f32 {
        self.num as f32 / self.den as f32
    }
}

impl Default for TimeBase {
    /// Defaults to 1/1 (i.e. whole seconds), since 0 in either field would
    /// make the timebase meaningless or cause a division by zero in `rescale`.
    fn default() -> Self {
        TimeBase { num: 1, den: 1 }
    }
}

/// A timestamp represents a point in time, expressed in a specific timebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Timestamp {
    /// The value of the timestamp in the units of the timebase.
    pub value: i64,

    /// The timebase associated with this timestamp.
    pub timebase: TimeBase,
}

impl Timestamp {
    /// Creates a new timestamp with the given value and timebase.
    pub const fn new(value: i64, timebase: TimeBase) -> Timestamp {
        Timestamp { value, timebase }
    }

    /// Rescales the timestamp to a new timebase.
    pub fn rescale(&self, new_timebase: TimeBase) -> Timestamp {
        if self.timebase == new_timebase {
            return *self;
        }

        let new_value = (self.value as i128 * new_timebase.num as i128 * self.timebase.den as i128)
            / (self.timebase.num as i128 * new_timebase.den as i128);

        Timestamp {
            value: new_value as i64,
            timebase: new_timebase,
        }
    }

    /// Converts the timestamp to seconds as a floating-point number.
    pub fn to_seconds(&self) -> f64 {
        self.value as f64 * self.timebase.as_f32() as f64
    }
}
