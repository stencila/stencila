TIDYHTML5_VERSION := 5.1.25

resources/tidy-html5-$(TIDYHTML5_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/htacg/tidy-html5/archive/$(TIDYHTML5_VERSION).tar.gz

build/requires/tidy-html5: resources/tidy-html5-$(TIDYHTML5_VERSION).tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/tidy-html5-$(TIDYHTML5_VERSION) build/requires/tidy-html5
	touch $@

TIDYHTML5_CMAKE_FLAGS :=
ifeq ($(OS), win)
TIDYHTML5_CMAKE_FLAGS += -G "MSYS Makefiles" -DCMAKE_C_FLAGS="-O2"
else
TIDYHTML5_CMAKE_FLAGS += -DCMAKE_C_FLAGS="-O2 -fPIC"
endif
build/requires/tidy-html5-built.flag: build/requires/tidy-html5
	cd build/requires/tidy-html5/build/cmake ;\
	  cmake $(TIDYHTML5_CMAKE_FLAGS) ../..
ifeq ($(OS), win)
	cd build/requires/tidy-html5/build/cmake ;\
		cmake --build . --config Release --target tidy-static
	# Under MSYS2 there are lots of multiple definition errors for localize symbols in the library
	objcopy --localize-symbols=requires/tidy-html5-localize-symbols.txt build/requires/tidy-html5/build/cmake/libtidys.a
else
	cd build/requires/tidy-html5/build/cmake ;\
		make
endif
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/tidy-html5/include
REQUIRES_LIB_DIRS += -Lbuild/requires/tidy-html5/build/cmake
REQUIRES_LIBS += tidys

requires-tidy-html5: build/requires/tidy-html5-built.flag
