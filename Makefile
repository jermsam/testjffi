.PHONY: help build run dev clean

help:
	@echo "JFFI App - Build Commands"
	@echo ""
	@echo "  make build PLATFORM=<platform>  - Build for platform"
	@echo "  make run PLATFORM=<platform>    - Run on platform"
	@echo "  make dev PLATFORM=<platform>    - Watch mode"
	@echo "  make clean                      - Clean build artifacts"
	@echo ""
	@echo "Available platforms: windows"
	@echo "Default platform: windows"

PLATFORM ?= windows

build:
	@jffi build --platform $(PLATFORM)

run:
	@jffi run --platform $(PLATFORM)

dev:
	@jffi dev --platform $(PLATFORM)

clean:
	@cargo clean
	@echo "✅ Cleaned build artifacts"
