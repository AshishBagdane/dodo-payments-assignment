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

use std::str::FromStr;

impl FromStr for TransactionType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
}

 impl TransactionType {
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
            "credit".parse::<TransactionType>().unwrap(),
            TransactionType::Credit
        );
        assert_eq!(
            "CREDIT".parse::<TransactionType>().unwrap(),
            TransactionType::Credit
        );
        assert_eq!(
            "debit".parse::<TransactionType>().unwrap(),
            TransactionType::Debit
        );
        assert_eq!(
            "transfer".parse::<TransactionType>().unwrap(),
            TransactionType::Transfer
        );
    }

    #[test]
    fn test_transaction_type_from_str_invalid() {
        let result = "invalid".parse::<TransactionType>();
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