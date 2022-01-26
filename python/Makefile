all: lint format test build docs

setup:
	python3 -m venv venv
	venv/bin/pip3 install -r requirements-dev.txt

install:
	python3 setup.py install

lint:
	venv/bin/python3 -m pylint stencila/schema
	venv/bin/python3 -m mypy stencila/schema --config-file .mypy.ini
	# cargo clippy

format:
	venv/bin/python3 -m black --exclude stencila/schema/types.py ./stencila ./*.py
	# cargo fmt

test:
	venv/bin/python3 -m tox

build:
	python3 setup.py sdist bdist_wheel
	# cargo build --release
.PHONY: build

docs:
	venv/bin/python3 -m pdoc --html --output-dir docs --force ./stencila/schema
	mv docs/schema/* docs
	rmdir docs/schema
.PHONY: docs

clean:
	rm -rf build dist .mypy_cache .coverage coverage.xml htmlcov .tox *.egg-info **/__pycache__
