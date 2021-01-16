all: format lint build

setup:
	python3 -m venv venv
	venv/bin/pip install -r requirements-dev.txt

format:
	cargo fmt
	venv/bin/black . --exclude /venv/

lint:
	touch src/*.rs && cargo clippy

build:
	cargo build --release
