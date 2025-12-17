use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::value_objects::Money;

/// Account entity representing a business account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub business_name: String,
    pub balance: Money,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// Create a new account with initial balance
    pub fn new(business_name: String, initial_balance: Money) -> Result<Self, DomainError> {
        Self::validate_business_name(&business_name)?;
        initial_balance.validate()?;

        Ok(Self {
            id: Uuid::new_v4(),
            business_name,
            balance: initial_balance,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Create account from database record
    pub fn from_db(
        id: Uuid,
        business_name: String,
        balance: Decimal,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            business_name,
            balance: Money::new(balance)?,
            created_at,
            updated_at,
        })
    }

    /// Credit (add) money to the account
    pub fn credit(&mut self, amount: Money) -> Result<(), DomainError> {
        amount.validate()?;

        self.balance = self.balance.add(amount)?;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Debit (subtract) money from the account
    pub fn debit(&mut self, amount: Money) -> Result<(), DomainError> {
        amount.validate()?;

        if !self.has_sufficient_balance(amount) {
            return Err(DomainError::InsufficientBalance {
                available: self.balance.to_string(),
                required: amount.to_string(),
            });
        }

        self.balance = self.balance.subtract(amount)?;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if account has sufficient balance
    pub fn has_sufficient_balance(&self, amount: Money) -> bool {
        self.balance >= amount
    }

    /// Get current balance as Decimal for database operations
    pub fn balance_as_decimal(&self) -> Decimal {
        self.balance.amount()
    }

    /// Validate business name
    fn validate_business_name(name: &str) -> Result<(), DomainError> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(DomainError::InvalidAccountState(
                "Business name cannot be empty".to_string(),
            ));
        }

        if trimmed.len() > 255 {
            return Err(DomainError::InvalidAccountState(
                "Business name cannot exceed 255 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Update business name
    pub fn update_business_name(&mut self, new_name: String) -> Result<(), DomainError> {
        Self::validate_business_name(&new_name)?;
        self.business_name = new_name;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_new_account() {
        let account = Account::new(
            "Test Business".to_string(),
            Money::new(dec!(1000.00)).unwrap(),
        )
            .unwrap();

        assert_eq!(account.business_name, "Test Business");
        assert_eq!(account.balance, Money::new(dec!(1000.00)).unwrap());
    }

    #[test]
    fn test_create_account_with_empty_name() {
        let result = Account::new("".to_string(), Money::new(dec!(0.00)).unwrap());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InvalidAccountState(_)
        ));
    }

    #[test]
    fn test_create_account_with_long_name() {
        let long_name = "a".repeat(256);
        let result = Account::new(long_name, Money::new(dec!(0.00)).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_credit_account() {
        let mut account = Account::new(
            "Test Business".to_string(),
            Money::new(dec!(100.00)).unwrap(),
        )
            .unwrap();

        account.credit(Money::new(dec!(50.00)).unwrap()).unwrap();
        assert_eq!(account.balance, Money::new(dec!(150.00)).unwrap());
    }

    #[test]
    fn test_debit_account_sufficient_balance() {
        let mut account = Account::new(
            "Test Business".to_string(),
            Money::new(dec!(100.00)).unwrap(),
        )
            .unwrap();

        account.debit(Money::new(dec!(50.00)).unwrap()).unwrap();
        assert_eq!(account.balance, Money::new(dec!(50.00)).unwrap());
    }

    #[test]
    fn test_debit_account_insufficient_balance() {
        let mut account = Account::new(
            "Test Business".to_string(),
            Money::new(dec!(100.00)).unwrap(),
        )
            .unwrap();

        let result = account.debit(Money::new(dec!(150.00)).unwrap());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InsufficientBalance { .. }
        ));
    }

    #[test]
    fn test_has_sufficient_balance() {
        let account = Account::new(
            "Test Business".to_string(),
            Money::new(dec!(100.00)).unwrap(),
        )
            .unwrap();

        assert!(account.has_sufficient_balance(Money::new(dec!(50.00)).unwrap()));
        assert!(account.has_sufficient_balance(Money::new(dec!(100.00)).unwrap()));
        assert!(!account.has_sufficient_balance(Money::new(dec!(150.00)).unwrap()));
    }

    #[test]
    fn test_update_business_name() {
        let mut account = Account::new(
            "Old Name".to_string(),
            Money::new(dec!(100.00)).unwrap(),
        )
            .unwrap();

        account.update_business_name("New Name".to_string()).unwrap();
        assert_eq!(account.business_name, "New Name");
    }

    #[test]
    fn test_update_business_name_invalid() {
        let mut account = Account::new(
            "Old Name".to_string(),
            Money::new(dec!(100.00)).unwrap(),
        )
            .unwrap();

        let result = account.update_business_name("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_credit_updates_timestamp() {
        let mut account = Account::new(
            "Test Business".to_string(),
            Money::new(dec!(100.00)).unwrap(),
        )
            .unwrap();

        let old_updated_at = account.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        account.credit(Money::new(dec!(50.00)).unwrap()).unwrap();
        assert!(account.updated_at > old_updated_at);
    }

    #[test]
    fn test_from_db() {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let updated_at = Utc::now();

        let account = Account::from_db(
            id,
            "Test Business".to_string(),
            dec!(100.00),
            created_at,
            updated_at,
        )
            .unwrap();

        assert_eq!(account.id, id);
        assert_eq!(account.business_name, "Test Business");
        assert_eq!(account.balance, Money::new(dec!(100.00)).unwrap());
    }
}