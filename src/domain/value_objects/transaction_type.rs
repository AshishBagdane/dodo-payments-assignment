use serde::{Deserialize, Serialize};
use std::fmt;

use crate::domain::errors::DomainError;

/// Type of transaction: Credit, Debit, or Transfer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// Credit: Add money to an account (from_account is None)
    Credit,
    /// Debit: Remove money from an account (to_account is None)
    Debit,
    /// Transfer: Move money between two accounts
    Transfer,
}

impl TransactionType {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.to_lowercase().as_str() {
            "credit" => Ok(Self::Credit),
            "debit" => Ok(Self::Debit),
            "transfer" => Ok(Self::Transfer),
            _ => Err(DomainError::InvalidTransactionType(format!(
                "Invalid transaction type: {}",
                s
            ))),
        }
    }

    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Credit => "credit",
            Self::Debit => "debit",
            Self::Transfer => "transfer",
        }
    }
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_type_from_str() {
        assert_eq!(
            TransactionType::from_str("credit").unwrap(),
            TransactionType::Credit
        );
        assert_eq!(
            TransactionType::from_str("CREDIT").unwrap(),
            TransactionType::Credit
        );
        assert_eq!(
            TransactionType::from_str("debit").unwrap(),
            TransactionType::Debit
        );
        assert_eq!(
            TransactionType::from_str("transfer").unwrap(),
            TransactionType::Transfer
        );
    }

    #[test]
    fn test_transaction_type_from_str_invalid() {
        let result = TransactionType::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_type_as_str() {
        assert_eq!(TransactionType::Credit.as_str(), "credit");
        assert_eq!(TransactionType::Debit.as_str(), "debit");
        assert_eq!(TransactionType::Transfer.as_str(), "transfer");
    }

    #[test]
    fn test_transaction_type_display() {
        assert_eq!(format!("{}", TransactionType::Credit), "credit");
        assert_eq!(format!("{}", TransactionType::Debit), "debit");
        assert_eq!(format!("{}", TransactionType::Transfer), "transfer");
    }
}