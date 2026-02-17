db:
	@chmod +x ./scripts/setup.sh
	@./scripts/setup.sh

down_migrations:
	@docker-compose down -v

run:
	@cargo run