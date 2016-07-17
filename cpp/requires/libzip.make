include ../shared.make

LIBZIP_VERSION := 1.1.2

resources/libzip-$(LIBZIP_VERSION).tar.gz:
	mkdir -p resources
	wget -q -O $@ http://www.nih.at/libzip/libzip-$(LIBZIP_VERSION).tar.gz

build/requires/libzip: resources/libzip-$(LIBZIP_VERSION).tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/libzip-$(LIBZIP_VERSION) $@
	touch $@
	
# Patch CMake config to compile static library
build/requires/libzip/lib/CMakeLists.txt: build/requires/libzip
	sed -i "s!ADD_LIBRARY(zip SHARED!ADD_LIBRARY(zip!" $@

LIBZIP_CMAKE_FLAGS :=
ifeq ($(OS), win)
	LIBZIP_CMAKE_FLAGS += -G "MSYS Makefiles"
else
	LIBZIP_CMAKE_FLAGS += -DCMAKE_C_FLAGS=-fPIC
endif
build/requires/libzip/build/lib/libzip.a: build/requires/libzip/lib/CMakeLists.txt
	cd build/requires/libzip ;\
	  mkdir -p build ;\
	  cd build ;\
	  cmake .. $(LIBZIP_CMAKE_FLAGS);\
	  cmake --build . --target zip
	touch $@

libzip: build/requires/libzip/build/lib/libzip.a
