EXECUTABLE := compiled/output_exec

RELEASE_OUTPUT := target/release/adan
DEBUG_OUTPUT := target/debug/adan

DOCKER_INSTALLER := util/docker.py
DOCKER_MOUNT := scripts/mount_docker.sh
DOCKER_CONTAINER_NAME := rust_llvm15

all:
	python3 $(DOCKER_INSTALLER) || python $(DOCKER_INSTALLER) || py $(DOCKER_INSTALLER)
	$(DOCKER_MOUNT)

compile:
	$(MAKE) all
	docker exec -it $(DOCKER_CONTAINER_NAME) cargo clean
	docker exec -it $(DOCKER_CONTAINER_NAME) cargo build
	docker exec -it $(DOCKER_CONTAINER_NAME) $(DEBUG_OUTPUT)

run:
	$(MAKE) all
	docker exec -it $(DOCKER_CONTAINER_NAME) $(DEBUG_OUTPUT)
	docker exec -it $(DOCKER_CONTAINER_NAME) $(EXECUTABLE)

debug:
	$(MAKE) compile
	$(MAKE) run