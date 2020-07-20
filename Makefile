all: setup lint format build

setup:
	npm install

lint:
	npm run lint

format:
	npm run format

build:
	npm run build

clean:
	npm run clean
