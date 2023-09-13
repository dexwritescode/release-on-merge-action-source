
.PHONY: help
help: ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: build
build: ## Compile the service
	cargo build

.PHONY: run
run: ## Compile and run the service
	cargo run

.PHONY: buildrelease
buildrelease: ## Compile the service in release mode
	cargo build --release

.PHONY: runrelease
runrelease: ## Compile and run the service in release mode
	cargo run --release
