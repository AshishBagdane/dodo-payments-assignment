use std::sync::Arc;
use crate::application::services::{AccountService, AuthService, TransactionService};

#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
    pub transaction_service: Arc<TransactionService>,
    pub auth_service: Arc<AuthService>,
}
