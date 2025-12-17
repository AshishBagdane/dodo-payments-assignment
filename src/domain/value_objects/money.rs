use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

use crate::domain::errors::DomainError;

/// Money value object representing monetary amounts with precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
}

impl Money {
    /// Create a new Money instance
    pub fn new(amount: Decimal) -> Result<Self, DomainError> {
        let money = Self { amount };
        money.validate()?;
        Ok(money)
    }

    /// Create Money from a string (useful for parsing user input)
    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        let amount = s
            .parse::<Decimal>()
            .map_err(|_| DomainError::InvalidAmount(format!("Invalid amount: {}", s)))?;
        Self::new(amount)
    }

    /// Get the amount as Decimal
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// Validate money constraints
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.amount.is_sign_negative() {
            return Err(DomainError::InvalidAmount(
                "Amount cannot be negative".to_string(),
            ));
        }

        // Check for reasonable precision (2 decimal places for currency)
        if self.amount.scale() > 2 {
            return Err(DomainError::InvalidAmount(
                "Amount cannot have more than 2 decimal places".to_string(),
            ));
        }

        // Check for reasonable maximum (avoid overflow)
        let max = Decimal::from(999_999_999_999_999i64); // ~999 trillion
        if self.amount > max {
            return Err(DomainError::InvalidAmount(
                "Amount exceeds maximum allowed value".to_string(),
            ));
        }

        Ok(())
    }

    /// Add two Money values
    pub fn add(self, other: Money) -> Result<Money, DomainError> {
        let result = self.amount + other.amount;
        Money::new(result)
    }

    /// Subtract two Money values
    pub fn subtract(self, other: Money) -> Result<Money, DomainError> {
        let result = self.amount - other.amount;
        Money::new(result)
    }

    /// Check if amount is zero
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// Check if amount is positive
    pub fn is_positive(&self) -> bool {
        self.amount > Decimal::ZERO
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.amount)
    }
}

impl Add for Money {
    type Output = Result<Money, DomainError>;

    fn add(self, other: Money) -> Self::Output {
        self.add(other)
    }
}

impl Sub for Money {
    type Output = Result<Money, DomainError>;

    fn sub(self, other: Money) -> Self::Output {
        self.subtract(other)
    }
}

impl From<Money> for Decimal {
    fn from(money: Money) -> Self {
        money.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_valid_money() {
        let money = Money::new(dec!(100.50)).unwrap();
        assert_eq!(money.amount(), dec!(100.50));
    }

    #[test]
    fn test_create_zero_money() {
        let money = Money::new(dec!(0.00)).unwrap();
        assert!(money.is_zero());
    }

    #[test]
    fn test_negative_money_fails() {
        let result = Money::new(dec!(-10.00));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidAmount(_)));
    }

    #[test]
    fn test_too_many_decimal_places_fails() {
        let result = Money::new(dec!(10.123));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_money() {
        let money1 = Money::new(dec!(100.00)).unwrap();
        let money2 = Money::new(dec!(50.50)).unwrap();
        let result = money1.add(money2).unwrap();
        assert_eq!(result.amount(), dec!(150.50));
    }

    #[test]
    fn test_subtract_money() {
        let money1 = Money::new(dec!(100.00)).unwrap();
        let money2 = Money::new(dec!(50.50)).unwrap();
        let result = money1.subtract(money2).unwrap();
        assert_eq!(result.amount(), dec!(49.50));
    }

    #[test]
    fn test_subtract_resulting_in_negative_fails() {
        let money1 = Money::new(dec!(50.00)).unwrap();
        let money2 = Money::new(dec!(100.00)).unwrap();
        let result = money1.subtract(money2);
        assert!(result.is_err());
    }

    #[test]
    fn test_money_display() {
        let money = Money::new(dec!(1234.56)).unwrap();
        assert_eq!(format!("{}", money), "1234.56");
    }

    #[test]
    fn test_money_from_string() {
        let money = Money::from_str("123.45").unwrap();
        assert_eq!(money.amount(), dec!(123.45));
    }

    #[test]
    fn test_money_from_invalid_string() {
        let result = Money::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_money_comparison() {
        let money1 = Money::new(dec!(100.00)).unwrap();
        let money2 = Money::new(dec!(50.00)).unwrap();
        let money3 = Money::new(dec!(100.00)).unwrap();

        assert!(money1 > money2);
        assert!(money2 < money1);
        assert_eq!(money1, money3);
    }

    #[test]
    fn test_is_positive() {
        let money1 = Money::new(dec!(100.00)).unwrap();
        let money2 = Money::new(dec!(0.00)).unwrap();

        assert!(money1.is_positive());
        assert!(!money2.is_positive());
    }

    #[test]
    fn test_money_exceeds_maximum() {
        let result = Money::new(Decimal::from(1_000_000_000_000_000i64));
        assert!(result.is_err());
    }
}