use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Offset};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RTC {
    base_datetime: DateTime<FixedOffset>,
    base_instant: Instant,
}

impl RTC {
    /// Creates a new RTC instance with the current system time.
    pub fn new() -> Self {
        let now = chrono::Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());
        Self {
            base_datetime: now,
            base_instant: Instant::now(),
        }
    }

    /// Set the simulated clock time with values from an RPC-style request
    pub fn set_time(
        &mut self,
        year: i32,
        mon: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        time_zone: &str,
    ) -> Result<(), String> {
        let tz = match time_zone.parse::<chrono_tz::Tz>() {
            Ok(tz) => tz,
            Err(_) => return Err(format!("Invalid timezone: {}", time_zone)),
        };

        let naive_date = NaiveDate::from_ymd_opt(year, mon, day)
            .ok_or("Invalid date")?;
        let naive_time = NaiveTime::from_hms_opt(hour, min, sec)
            .ok_or("Invalid time")?;

        let naive = naive_date.and_time(naive_time);
        let datetime = tz.from_local_datetime(&naive)
            .single()
            .ok_or("Ambiguous or nonexistent local time")?;

        self.base_datetime = datetime.with_timezone(&datetime.offset().fix());
        self.base_instant = Instant::now();
        Ok(())
    }

    /// Get the current simulated time
    #[allow(dead_code)]
    pub fn now(&self) -> DateTime<FixedOffset> {
        let elapsed = self.base_instant.elapsed();
        self.base_datetime + chrono::Duration::from_std(elapsed).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_set_and_get_time_basic() {
        let mut rtc = RTC::new();

        rtc.set_time(2025, 5, 6, 18, 44, 31, "America/Costa_Rica").unwrap();

        thread::sleep(Duration::from_millis(1000));

        let now = rtc.now();

        assert_eq!(now.year(), 2025);
        assert_eq!(now.month(), 5);
        assert_eq!(now.day(), 6);
        assert_eq!(now.hour(), 18);
        assert_eq!(now.minute(), 44);
        assert!(now.second() >= 32);
    }

    #[test]
    fn test_time_advances() {
        let mut rtc = RTC::new();
        rtc.set_time(2025, 1, 1, 0, 0, 0, "UTC").unwrap();

        let first = rtc.now();
        thread::sleep(Duration::from_millis(500));
        let later = rtc.now();

        assert!(later > first);
    }

    #[test]
    fn test_invalid_timezone() {
        let mut rtc = RTC::new();
        let result = rtc.set_time(2025, 1, 1, 0, 0, 0, "Invalid/Zone");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid timezone"));
    }
}