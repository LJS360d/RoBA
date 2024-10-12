ifeq ($(OS),Windows_NT)
# Windows specific
	EXE := .exe
	DEL := del /f
	SET_ENV := set
	SEP := &
else
# Other shells
	EXE :=
	DEL := rm -f
	SET_ENV :=
	SEP := ;
endif
MODULE_NAME   := $(shell go list -m)

BUILD_DIR 		= bin
BINARY_NAME 	= $(MODULE_NAME)
BUILDPATH 		= $(BUILD_DIR)/$(BINARY_NAME)$(EXE)
MAIN_PACKAGE 	= ./cmd/desktop

WASM_BINARY_NAME 	= $(MODULE_NAME).wasm
WASM_BUILDPATH 		= $(BUILD_DIR)/$(WASM_BINARY_NAME)
WASM_MAIN_PACKAGE 	= ./cmd/web

all: build-wasm

build:
	go build -o $(BUILDPATH) $(MAIN_PACKAGE)

build-wasm:
	$(SET_ENV) GOOS=js$(SEP) $(SET_ENV) GOARCH=wasm$(SEP) go build -o $(WASM_BUILDPATH) $(WASM_MAIN_PACKAGE)

test:
	go test -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html

lint:
	go fmt ./...

clean:
	go clean
	$(DEL) $(BUILDPATH)
	$(DEL) $(WASM_BUILDPATH)

run: build
	./$(BUILDPATH)

.PHONY: all build build-wasm test lint clean run
