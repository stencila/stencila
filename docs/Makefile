.PHONY: upload build cpp guide

all: build

deploy:
	ghp-import -p build

build: cpp guide
	rm -rf build
	mkdir build
	cp -f index.html build
	cp -fr style build
	cp -fr cpp/html build/cpp
	cp -fr guide/html build/guide
	cp -f CNAME build

cpp:
	make -C cpp all

guide:
	make -C guide all
	
clean:
	make -C cpp clean
	make -C guide clean

