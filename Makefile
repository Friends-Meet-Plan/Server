db:
	@chmod +x ./scripts/setup.sh
	@./scripts/setup.sh

server:
	@./scripts/setup.sh & cargo run