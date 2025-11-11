# --------- CAPPUCINA MAKEFILE --------- #
# Used for testing and compiling code easier, with the commands below
# Under the MIT license, to see more please refer to LICENSE.md
# --------- WRITTEN BY @NVTTLES --------- #

OS := $(shell python3 util/detect_os.py 2>/dev/null || python detect_os.py)
SRC := src/main.c
OUT := build/main

ifeq ($(OS),posix)
	CC := gcc
	RM := rm -f
	MKDIR := mkdir -p
	COMPILE_FLAGS := -Wall -O2
else ifeq ($(OS),nt)
	CC := gcc
	RM := del
	MKDIR := mkdir
	OUT := build\\main.exe
	COMPILE_FLAGS := -Wall -O2
else
	$(error Unknown or unsupported OS detected: $(OS))
endif

compile:
	@chmod +x ./scripts/mount-docker.sh; ./scripts/mount_docker.sh
	@cargo clean
	@cargo build

run:
	@echo "Running compiled program with args: $(ARGS)"
	@./output_exec

clean:
	@echo "Cleaning Cargo build..."
	@cargo clean

all: compile

.PHONY: all compile clean