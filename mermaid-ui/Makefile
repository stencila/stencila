install:
ifdef CI
	cd .. && npm ci --ignore-scripts
else
	cd .. && npm install --ignore-scripts
endif

build: install
	npm run build

clean:
	rm -rf ./dist