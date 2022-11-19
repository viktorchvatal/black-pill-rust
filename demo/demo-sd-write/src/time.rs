use embedded_sdmmc::{Timestamp, TimeSource};
use pcf8563::DateTime;

pub struct ClockData {
    timestamp: Timestamp,
    day_of_week: u8,
}

const ZERO_TIMESTAMP: Timestamp = Timestamp {
    year_since_1970: 0,
    zero_indexed_month: 0,
    zero_indexed_day: 0,
    hours: 0,
    minutes: 0,
    seconds: 0,
};

impl Default for ClockData {
    fn default() -> Self {
        Self {
            timestamp: ZERO_TIMESTAMP,
            day_of_week: 0,
        }
    }
}

impl TimeSource for ClockData {
    fn get_timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

impl ClockData {
    pub fn set_from_pcf8563(&mut self, time: DateTime) {
        self.timestamp = pcf8563_date_time_to_timestamp(time);
        self.day_of_week = time.weekday;
    }

    pub fn reset_to_default(&mut self) {
        self.timestamp = ZERO_TIMESTAMP;
        self.day_of_week = 0;
    }

    pub fn year(&self) -> u16 {
        self.timestamp.year_since_1970 as u16 + 1970
    }

    pub fn month(&self) -> u8 {
        self.timestamp.zero_indexed_month + 1
    }

    pub fn day(&self) -> u8 {
        self.timestamp.zero_indexed_day + 1
    }

    #[allow(dead_code)]
    pub fn week_day(&self) -> u8 {
        self.day_of_week
    }

    pub fn hours(&self) -> u8 {
        self.timestamp.hours
    }

    pub fn minutes(&self) -> u8 {
        self.timestamp.minutes
    }

    pub fn seconds(&self) -> u8 {
        self.timestamp.seconds
    }
}

fn pcf8563_date_time_to_timestamp(time: DateTime) -> Timestamp {
    Timestamp {
        year_since_1970: ((time.year as u32) + 2000 - 1970) as u8,
        zero_indexed_month: time.month - 1,
        zero_indexed_day: time.day - 1,
        hours: time.hours,
        minutes: time.minutes,
        seconds: time.seconds,
    }
}