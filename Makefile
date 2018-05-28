all: setup build

setup:
	npm install

run:
	npm start

build:
	npm run build
.PHONY: build

clean:
	npm run clean
