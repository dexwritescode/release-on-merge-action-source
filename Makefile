
.PHONY: help
help: ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: build
build: ## Build
	cargo build

.PHONY: run
run: ## Build and run
	cargo run

.PHONY: buildrelease
buildrelease: ## Build in release mode
	cargo build --release

.PHONY: runrelease
runrelease: ## Build and run in release mode
	cargo run --release

.PHONY: dockerbuild
dockerbuild: ## Build in docker
	docker build -t release-on-merge-action .

.PHONY: dockerrun
dockerrun: ## Build and run in docker
	docker build -t release-on-merge-action . && docker run -it release-on-merge-action