use axum::http::StatusCode;
use chrono::{Datelike, NaiveDate};

use crate::AppError;

pub enum BirthDateError {
    InvalidFormat,
    Underage,
}

impl From<BirthDateError> for AppError {
    fn from(err: BirthDateError) -> Self {
        let message = err.to_string();
        match err {
            BirthDateError::InvalidFormat => {
                AppError::new(message, "INVALID_DATE_FORMAT", StatusCode::BAD_REQUEST)
            }
            BirthDateError::Underage => {
                AppError::new(message, "UNDERAGE_USER", StatusCode::BAD_REQUEST)
            }
        }
    }
}

impl std::fmt::Display for BirthDateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BirthDateError::InvalidFormat => write!(f, "Birth date must be in YYYY-MM-DD format"),
            BirthDateError::Underage => write!(f, "User must be at least 13 years old"),
        }
    }
}

pub struct BirthDate(String);

impl BirthDate {
    pub fn new(date: impl Into<String>) -> Result<Self, BirthDateError> {
        let date = date.into();
        let birth_date = BirthDate(date);
        birth_date.validate()?;
        Ok(birth_date)
    }
    fn years_between(start: NaiveDate, end: NaiveDate) -> i32 {
        let mut years = end.year() - start.year();
        // If end month/day is before start month/day, subtract one year
        if (end.month(), end.day()) < (start.month(), start.day()) {
            years -= 1;
        }
        years
    }

    fn validate(&self) -> Result<(), BirthDateError> {
        let date = chrono::NaiveDate::parse_from_str(&self.0, "%Y-%m-%d")
            .map_err(|_| BirthDateError::InvalidFormat)?;

        let today = chrono::Utc::now().date_naive();

        let age = Self::years_between(date, today);
        if age < 13 {
            return Err(BirthDateError::Underage);
        }

        Ok(())
    }

    pub fn as_date(&self) -> NaiveDate {
        chrono::NaiveDate::parse_from_str(&self.0, "%Y-%m-%d").unwrap()
    }
}
