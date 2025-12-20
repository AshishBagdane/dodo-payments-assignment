.PHONY: help build up down logs clean test db-reset

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the application
	cargo build --release

up: ## Start all services with docker-compose
	@if [ ! -f .env ]; then \
		echo "Creating .env from .env.example"; \
		cp .env.example .env; \
	fi
	docker-compose up -d

down: ## Stop all services
	docker-compose down

logs: ## Show logs from all services
	docker-compose logs -f

logs-app: ## Show logs from application only
	docker-compose logs -f app

logs-db: ## Show logs from database only
	docker-compose logs -f postgres

clean: ## Remove all containers, volumes, and build artifacts
	docker-compose down -v
	cargo clean

test: ## Run tests
	cargo test

db-reset: ## Reset database (WARNING: destroys all data)
	docker-compose down -v
	docker-compose up -d postgres
	sleep 5
	docker-compose up liquibase

dev: ## Start services for local development (DB only)
	docker-compose up -d postgres
	@echo "Waiting for PostgreSQL to be ready..."
	@sleep 5
	docker-compose up liquibase
	@echo "Database ready! Run 'cargo run' to start the application"

check: ## Check code compiles
	cargo check

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy linter
	cargo clippy -- -D warnings

sqlx-prepare: ## Prepare SQLx for offline compilation
	cargo sqlx prepare -- --lib --tests

sqlx-check: ## Verify SQLx queries without database
	cargo sqlx prepare --check -- --lib