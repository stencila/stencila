all: regen lint format test build docs

setup:
	pip3 install -r requirements-dev.txt

install:
	python3 setup.py install

regen:
	npm run build:python

lint:
	python3 -m pylint stencila/schema
	python3 -m mypy stencila/schema --config-file .mypy.ini

format:
	python3 -m black --exclude stencila/schema/types.py ./stencila ./*.py

test:
	python3 -m tox

build:
	python3 setup.py sdist bdist_wheel
.PHONY: build

docs:
	python3 -m pdoc --html --output-dir docs --force ./stencila/schema
	mv docs/schema/* docs
	rmdir docs/schema
.PHONY: docs

clean:
	rm -rf build dist .mypy_cache .coverage coverage.xml htmlcov .tox *.egg-info **/__pycache__
