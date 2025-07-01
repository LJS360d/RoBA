.PHONY: all build run clean test deps
ifeq ($(OS),Windows_NT)
	EXE := .exe
else
	EXE :=
endif
# Go parameters
GOCMD=go
BUILD_TAGS=release
GOBUILD=$(GOCMD) build -tags $(BUILD_TAGS)
GOCLEAN=$(GOCMD) clean
GOTEST=$(GOCMD) test
GOGET=$(GOCMD) get
GORUN=$(GOCMD) run

# Binary name
BINARY_NAME=GoBA$(EXE)
BINARY_UNIX=$(BINARY_NAME)

# All targets
all: build

$(BINARY_NAME):
	$(GOBUILD) -o $(BINARY_NAME) main.go

# Build the application
build:
	$(GOBUILD) -o $(BINARY_NAME) main.go

run: BUILD_TAGS=debug
run: build
	./$(BINARY_NAME) -rom=test/emerald.gba

# Clean the binary
clean:
	$(GOCLEAN)
	rm -f $(BINARY_NAME)
	rm -f $(BINARY_UNIX)

# Run tests
test:
	$(GOTEST) -v ./...

# Get dependencies
deps:
	$(GOGET)

