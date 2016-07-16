CMARK_VERSION := 0.25.2

resources/cmark-$(CMARK_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/jgm/cmark/archive/$(CMARK_VERSION).tar.gz

build/requires/cmark: resources/cmark-$(CMARK_VERSION).tar.gz
	mkdir -p build/requires
	tar xzf $< -C build/requires
	rm -rf $@
	mv build/requires/cmark-$(CMARK_VERSION) $@
	touch $@

CMARK_CMAKE_FLAGS := 
ifeq ($(OS), win)
	CMARK_CMAKE_FLAGS += -G "MSYS Makefiles"
else
	CMARK_CMAKE_FLAGS += -DCMAKE_C_FLAGS=-fPIC
endif
build/requires/cmark/build/src/libcmark.a: build/requires/cmark
	mkdir -p $</build
	cd $</build ;\
		cmake .. $(CMARK_CMAKE_FLAGS) ;\
		make libcmark_static

requires-cmark: build/requires/cmark/build/src/libcmark.a

REQUIRES_INC_DIRS += -Ibuild/requires/cmark/src -Ibuild/requires/cmark/build/src
REQUIRES_LIB_DIRS += -Lbuild/requires/cmark/build/src
REQUIRES_LIBS += cmark
