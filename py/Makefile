all: regen lint test build

setup:
	pip3 install --upgrade -r requirements-dev.txt

install:
	python3 setup.py install

regen:
	npm run build:py

lint:
	pylint stencila/schema --max-line-length=120
	mypy stencila/schema --ignore-missing-imports

test:
	tox

build:
	python3 setup.py sdist bdist_wheel

clean:
	rm -rf build dist .mypy_cache .coverage coverage.xml htmlcov .tox *.egg-info **/__pycache__
