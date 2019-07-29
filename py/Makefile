all: setup clean regen lint test build

setup:
	pip3 install --user --upgrade -r requirements-dev.txt

regen:
	npm run build:py

lint:
	pylint stencila/schema
	mypy stencila/schema

test:
	tox

build:
	python3 setup.py sdist bdist_wheel

clean:
	rm -rf build .coverage coverage.xml *.egg-info .tox **/__pycache__
