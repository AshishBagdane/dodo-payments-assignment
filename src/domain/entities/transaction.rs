use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::value_objects::{Money, TransactionType};

/// Transaction entity representing a financial transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub amount: Money,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    /// Create a new credit transaction (money added to account)
    pub fn new_credit(
        to_account_id: Uuid,
        amount: Money,
        idempotency_key: Option<String>,
    ) -> Result<Self, DomainError> {
        amount.validate()?;

        if !amount.is_positive() {
            return Err(DomainError::InvalidAmount(
                "Transaction amount must be positive".to_string(),
            ));
        }

        if let Some(ref key) = idempotency_key {
            Self::validate_idempotency_key(key)?;
        }

        Ok(Self {
            id: Uuid::new_v4(),
            transaction_type: TransactionType::Credit,
            from_account_id: None,
            to_account_id: Some(to_account_id),
            amount,
            idempotency_key,
            created_at: Utc::now(),
        })
    }

    /// Create a new debit transaction (money removed from account)
    pub fn new_debit(
        from_account_id: Uuid,
        amount: Money,
        idempotency_key: Option<String>,
    ) -> Result<Self, DomainError> {
        amount.validate()?;

        if !amount.is_positive() {
            return Err(DomainError::InvalidAmount(
                "Transaction amount must be positive".to_string(),
            ));
        }

        if let Some(ref key) = idempotency_key {
            Self::validate_idempotency_key(key)?;
        }

        Ok(Self {
            id: Uuid::new_v4(),
            transaction_type: TransactionType::Debit,
            from_account_id: Some(from_account_id),
            to_account_id: None,
            amount,
            idempotency_key,
            created_at: Utc::now(),
        })
    }

    /// Create a new transfer transaction (money moved between accounts)
    pub fn new_transfer(
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: Money,
        idempotency_key: Option<String>,
    ) -> Result<Self, DomainError> {
        amount.validate()?;

        if !amount.is_positive() {
            return Err(DomainError::InvalidAmount(
                "Transaction amount must be positive".to_string(),
            ));
        }

        if from_account_id == to_account_id {
            return Err(DomainError::SelfTransferNotAllowed);
        }

        if let Some(ref key) = idempotency_key {
            Self::validate_idempotency_key(key)?;
        }

        Ok(Self {
            id: Uuid::new_v4(),
            transaction_type: TransactionType::Transfer,
            from_account_id: Some(from_account_id),
            to_account_id: Some(to_account_id),
            amount,
            idempotency_key,
            created_at: Utc::now(),
        })
    }

    /// Reconstruct transaction from database
    pub fn from_db(
        id: Uuid,
        transaction_type: TransactionType,
        from_account_id: Option<Uuid>,
        to_account_id: Option<Uuid>,
        amount: Money,
        idempotency_key: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let transaction = Self {
            id,
            transaction_type,
            from_account_id,
            to_account_id,
            amount,
            idempotency_key,
            created_at,
        };

        transaction.validate_invariants()?;
        Ok(transaction)
    }

    /// Validate transaction invariants based on type
    fn validate_invariants(&self) -> Result<(), DomainError> {
        match self.transaction_type {
            TransactionType::Credit => {
                if self.from_account_id.is_some() {
                    return Err(DomainError::InvalidTransactionType(
                        "Credit transaction cannot have from_account".to_string(),
                    ));
                }
                if self.to_account_id.is_none() {
                    return Err(DomainError::InvalidTransactionType(
                        "Credit transaction must have to_account".to_string(),
                    ));
                }
            }
            TransactionType::Debit => {
                if self.from_account_id.is_none() {
                    return Err(DomainError::InvalidTransactionType(
                        "Debit transaction must have from_account".to_string(),
                    ));
                }
                if self.to_account_id.is_some() {
                    return Err(DomainError::InvalidTransactionType(
                        "Debit transaction cannot have to_account".to_string(),
                    ));
                }
            }
            TransactionType::Transfer => {
                if self.from_account_id.is_none() {
                    return Err(DomainError::InvalidTransactionType(
                        "Transfer transaction must have from_account".to_string(),
                    ));
                }
                if self.to_account_id.is_none() {
                    return Err(DomainError::InvalidTransactionType(
                        "Transfer transaction must have to_account".to_string(),
                    ));
                }
                if self.from_account_id == self.to_account_id {
                    return Err(DomainError::SelfTransferNotAllowed);
                }
            }
        }

        Ok(())
    }

    /// Validate idempotency key format
    fn validate_idempotency_key(key: &str) -> Result<(), DomainError> {
        if key.trim().is_empty() {
            return Err(DomainError::InvalidAmount(
                "Idempotency key cannot be empty".to_string(),
            ));
        }

        if key.len() > 255 {
            return Err(DomainError::InvalidAmount(
                "Idempotency key cannot exceed 255 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if this is a credit transaction
    pub fn is_credit(&self) -> bool {
        matches!(self.transaction_type, TransactionType::Credit)
    }

    /// Check if this is a debit transaction
    pub fn is_debit(&self) -> bool {
        matches!(self.transaction_type, TransactionType::Debit)
    }

    /// Check if this is a transfer transaction
    pub fn is_transfer(&self) -> bool {
        matches!(self.transaction_type, TransactionType::Transfer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_credit_transaction() {
        let account_id = Uuid::new_v4();
        let amount = Money::new(dec!(100.00)).unwrap();

        let tx = Transaction::new_credit(
            account_id,
            amount,
            Some("idempotency-key-123".to_string()),
        )
            .unwrap();

        assert_eq!(tx.transaction_type, TransactionType::Credit);
        assert_eq!(tx.to_account_id, Some(account_id));
        assert_eq!(tx.from_account_id, None);
        assert_eq!(tx.amount, amount);
        assert!(tx.is_credit());
    }

    #[test]
    fn test_create_debit_transaction() {
        let account_id = Uuid::new_v4();
        let amount = Money::new(dec!(50.00)).unwrap();

        let tx = Transaction::new_debit(
            account_id,
            amount,
            None,
        )
            .unwrap();

        assert_eq!(tx.transaction_type, TransactionType::Debit);
        assert_eq!(tx.from_account_id, Some(account_id));
        assert_eq!(tx.to_account_id, None);
        assert!(tx.is_debit());
    }

    #[test]
    fn test_create_transfer_transaction() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let amount = Money::new(dec!(75.00)).unwrap();

        let tx = Transaction::new_transfer(
            from_id,
            to_id,
            amount,
            Some("transfer-key".to_string()),
        )
            .unwrap();

        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.from_account_id, Some(from_id));
        assert_eq!(tx.to_account_id, Some(to_id));
        assert!(tx.is_transfer());
    }

    #[test]
    fn test_transfer_to_self_fails() {
        let account_id = Uuid::new_v4();
        let amount = Money::new(dec!(100.00)).unwrap();

        let result = Transaction::new_transfer(
            account_id,
            account_id,
            amount,
            None,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::SelfTransferNotAllowed
        ));
    }

    #[test]
    fn test_zero_amount_fails() {
        let account_id = Uuid::new_v4();
        let amount = Money::new(dec!(0.00)).unwrap();

        let result = Transaction::new_credit(account_id, amount, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_idempotency_key_empty() {
        let result = Transaction::validate_idempotency_key("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_idempotency_key_too_long() {
        let long_key = "a".repeat(256);
        let result = Transaction::validate_idempotency_key(&long_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_db_validates_invariants() {
        let from_id = Uuid::new_v4();
        let amount = Money::new(dec!(100.00)).unwrap();

        // Invalid: Credit with from_account
        let result = Transaction::from_db(
            Uuid::new_v4(),
            TransactionType::Credit,
            Some(from_id),
            Some(Uuid::new_v4()),
            amount,
            None,
            Utc::now(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_credit_transaction_type_helpers() {
        let account_id = Uuid::new_v4();
        let amount = Money::new(dec!(100.00)).unwrap();

        let tx = Transaction::new_credit(account_id, amount, None).unwrap();

        assert!(tx.is_credit());
        assert!(!tx.is_debit());
        assert!(!tx.is_transfer());
    }

    #[test]
    fn test_debit_transaction_type_helpers() {
        let account_id = Uuid::new_v4();
        let amount = Money::new(dec!(100.00)).unwrap();

        let tx = Transaction::new_debit(account_id, amount, None).unwrap();

        assert!(!tx.is_credit());
        assert!(tx.is_debit());
        assert!(!tx.is_transfer());
    }

    #[test]
    fn test_transfer_transaction_type_helpers() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let amount = Money::new(dec!(100.00)).unwrap();

        let tx = Transaction::new_transfer(from_id, to_id, amount, None).unwrap();

        assert!(!tx.is_credit());
        assert!(!tx.is_debit());
        assert!(tx.is_transfer());
    }
}