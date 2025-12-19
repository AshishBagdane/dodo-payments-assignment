use dodo_payments_assignment::domain::entities::Account;
use dodo_payments_assignment::domain::repositories::AccountRepository;
use dodo_payments_assignment::domain::value_objects::Money;
use dodo_payments_assignment::infrastructure::config::Config;
use dodo_payments_assignment::infrastructure::database::{create_pool, PostgresAccountRepository};
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_account_repository_crud() {
    // 1. Setup configuration and connection pool
    // Force localhost for tests to ensure we connect to the exposed port, ignoring potentially 
    // container-internal hostnames from .env (e.g. dodo-payments-db)
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    let repository = PostgresAccountRepository::new(pool);

    // 2. Create Account
    let initial_balance = Money::new(dec!(100.00)).unwrap();
    let new_account = Account::new("Test Corp".to_string(), initial_balance.clone())
        .expect("Failed to create account entity");

    let created = repository.create(&new_account).await.expect("Failed to persist account");
    assert_eq!(created.business_name, "Test Corp");
    assert_eq!(created.balance, initial_balance);
    assert!(created.deleted_at.is_none());

    // 3. Find by ID
    let fetched = repository.find_by_id(created.id).await.expect("Failed to find account");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.business_name, created.business_name);

    // 4. Update Balance
    let new_balance = Money::new(dec!(200.00)).unwrap();
    repository.update_balance(created.id, new_balance.clone()).await.expect("Failed to update balance");
    
    let updated = repository.find_by_id(created.id).await.expect("Failed to find updated account");
    assert_eq!(updated.balance, new_balance);

    // 5. Update Business Name
    repository.update_business_name(created.id, "Updated Corp".to_string()).await.expect("Failed to update name");
    
    let updated_name = repository.find_by_id(created.id).await.expect("Failed to find updated account");
    assert_eq!(updated_name.business_name, "Updated Corp");

    // 6. List
    let list = repository.list(10, 0).await.expect("Failed to list accounts");
    assert!(list.iter().any(|a| a.id == created.id));

    // 7. Soft Delete
    repository.delete(created.id).await.expect("Failed to delete account");

    // 8. Verify Deletion
    let find_result = repository.find_by_id(created.id).await;
    assert!(find_result.is_err()); // Should be NotFound

    let exists_result = repository.exists(created.id).await.expect("Failed to check existence");
    assert!(!exists_result); // Should be false
}
