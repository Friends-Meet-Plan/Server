db:
	@chmod +x ./scripts/setup.sh
	@./scripts/setup.sh

server:
	@cargo run