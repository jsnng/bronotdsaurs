#![allow(dead_code)]
use crate::tds::prelude::*;
#[cfg(feature = "chrono")]
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

/// `DOY_MD[doy]` = (month, day) for day-of-year in the civil calendar.
/// doy 0 = March 1 (Hinnant's civil year starts at March).
const DOY_MD: [(u8, u8); 366] = {
    const MONTHS: [(u8, i32); 12] = [
        (3, 0), (4, 31), (5, 61), (6, 92), (7, 122), (8, 153),
        (9, 184), (10, 214), (11, 245), (12, 275), (1, 306), (2, 337),
    ];
    let mut table = [(0u8, 0u8); 366];
    let mut mi = 0usize;
    while mi < 12 {
        let (month, start) = MONTHS[mi];
        let end = if mi + 1 < 12 { MONTHS[mi + 1].1 } else { 366 };
        let mut doy = start;
        while doy < end {
            table[doy as usize] = (month, (doy - start + 1) as u8);
            doy += 1;
        }
        mi += 1;
    }
    table
};

/// Adapted from Howard Hinnant's `civil_from_days` algorithm
/// Returns year/month/day triple in the civil calendar.
/// # Note:
/// - uses TDS days (from 1900-01-01)
#[cfg_attr(kani, kani::ensures(|dt: &(i32, u8, u8)| {
    dt.1 >= 1 && dt.1 <= 12 &&
    dt.2 >= 1 && dt.2 <= 31
}))]
fn civil_from_days(z: i32) -> (i32, u8, u8) {
    let mut z = z;
    z += 693901; // = 719468 (Hinnant's Unix epoch offset) − 25567 (TDS epoch offset)
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    unsafe { core::hint::assert_unchecked((0..=146096).contains(&doe)) }
    let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
    unsafe { core::hint::assert_unchecked((0..=399).contains(&yoe)) }
    let y = yoe + era * 400;
    let doy = doe - (365*yoe + yoe/4 - yoe/100);
    unsafe { core::hint::assert_unchecked((0..=365).contains(&doy)) }
    let (m, d) = unsafe { *DOY_MD.get_unchecked(doy as usize) };
    let y = y + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

#[derive(Debug, Copy, Clone)]
pub struct SmallDateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
}

impl SmallDateTime {
    pub fn new(bytes: [u8; 4]) -> Self {
        let days: u16 = r_u16_le(&bytes, 0);
        let mins: u16 = r_u16_le(&bytes, 2);
        let (year, month, day) = civil_from_days(days as i32);
        Self {
            year,
            month,
            day,
            hour: (mins / 60) as u8,
            min: (mins % 60) as u8,
        }
    }
    #[inline]
    pub fn min(&self) -> u8 { self.min }
    #[inline]
    pub fn hour(&self) -> u8 { self.hour }
    #[inline]
    pub fn day(&self) -> u8 { self.day }
    #[inline]
    pub fn month(&self) -> u8 { self.month }
    #[inline]
    pub fn year(&self) -> i32 { self.year }
}

#[derive(Debug, Copy, Clone)]
pub struct DateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
    second: u8,
    milliseconds: u16,
}

impl DateTime {
    pub fn new(bytes: [u8; 8]) -> Self {
        let days: i32 = r_i32_le(&bytes, 0);
        let ticks: u32 = r_u32_le(&bytes, 4);
        let (year, month, day) = civil_from_days(days);
        let seconds: u32 = ticks / 300;
        let milliseconds: u16 = ((ticks % 300) * 1000 / 300) as u16;
        unsafe { core::hint::assert_unchecked((0..=999).contains(&milliseconds)) }
        Self {
            year,
            month,
            day,
            hour: (seconds / 3600) as u8,
            min: ((seconds % 3600) / 60) as u8,
            second: (seconds % 60) as u8,
            milliseconds,
        }
    }
    #[inline]
    pub fn milliseconds(&self) -> u16 { self.milliseconds }
    #[inline]
    pub fn second(&self) -> u8 { self.second }
    #[inline]
    pub fn min(&self) -> u8 { self.min }
    #[inline]
    pub fn hour(&self) -> u8 { self.hour }
    #[inline]
    pub fn day(&self) -> u8 { self.day }
    #[inline]
    pub fn month(&self) -> u8 { self.month }
    #[inline]
    pub fn year(&self) -> i32 { self.year }
}

impl core::fmt::Display for SmallDateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:04}-{:02}-{:02} {:02}:{:02}", self.year, self.month, self.day, self.hour, self.min)
    }
}

impl core::fmt::Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}", self.year, self.month, self.day, self.hour, self.min, self.second, self.milliseconds)
    }
}

#[cfg(feature = "chrono")]
impl From<SmallDateTime> for NaiveDateTime {
    fn from(dt: SmallDateTime) -> Self {
        let date = NaiveDate::from_ymd_opt(dt.year, dt.month.into(), dt.day.into()).unwrap();
        let time = NaiveTime::from_hms_opt(dt.hour().into(), dt.min().into(), 0).unwrap();
        NaiveDateTime::new(date, time)
    }
}

#[cfg(feature = "chrono")]
impl From<DateTime> for NaiveDateTime {
    fn from(dt: DateTime) -> Self {
        let date = NaiveDate::from_ymd_opt(dt.year, dt.month.into(), dt.day.into()).unwrap();
        let time = NaiveTime::from_hms_milli_opt(dt.hour().into(), dt.min().into(), dt.second().into(), dt.milliseconds().into()).unwrap();
        NaiveDateTime::new(date, time)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_civil_from_days_small_datetime() {
        let days: u16 = kani::any();
        let (_, m, d) = civil_from_days(days as i32);
        assert!(m >= 1 && m <= 12);
        assert!(d >= 1 && d <= 31);
    }

    #[kani::proof]
    fn verify_civil_from_days_datetime() {
        let days: i32 = kani::any();
        kani::assume(days >= -53690 && days <= 2932896);
        let (_, m, d) = civil_from_days(days);
        assert!(m >= 1 && m <= 12);
        assert!(d >= 1 && d <= 31);
    }
}
