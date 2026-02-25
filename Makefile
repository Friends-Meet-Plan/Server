db:
	@chmod +x ./scripts/setup.sh
	@./scripts/setup.sh

run:
	@cargo run --bin Server