.PHONY: upload build guide

all: build

deploy:
	ghp-import -m "Updated documentation" -p build

build: guide
	rm -rf build
	mkdir build
	cp -f index.html build
	cp -fr style build
	cp -fr guide/html build/guide
	cp -f CNAME build

guide:
	make -C guide all
	
clean:
	make -C guide clean
