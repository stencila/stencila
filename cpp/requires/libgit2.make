# Stencila Makefile for [libgit2](http://libgit2.github.com/)

LIBGIT2_VERSION := 0.20.0

libgit2-$(LIBGIT2_VERSION).zip:
	wget -O libgit2-$(LIBGIT2_VERSION).zip https://github.com/libgit2/libgit2/archive/v$(LIBGIT2_VERSION).zip

libgit2-$(LIBGIT2_VERSION): libgit2-$(LIBGIT2_VERSION).zip
	unzip libgit2-$(LIBGIT2_VERSION).zip
	touch libgit2-$(LIBGIT2_VERSION)

libgit2-$(LIBGIT2_VERSION)/build/libgit2.a: libgit2-$(LIBGIT2_VERSION)
	cd libgit2-$(LIBGIT2_VERSION);\
	mkdir -p build;\
	cd build;\
	cmake .. -DCMAKE_C_FLAGS=-fPIC -DBUILD_SHARED_LIBS=OFF;\
	cmake --build .

include/git2.h: libgit2-$(LIBGIT2_VERSION)
	ln -sfT ../libgit2-$(LIBGIT2_VERSION)/include/git2.h include/git2.h
	ln -sfT ../libgit2-$(LIBGIT2_VERSION)/include/git2 include/git2
	touch include/git2

lib/libgit2.a: libgit2-$(LIBGIT2_VERSION)/build/libgit2.a
	ln -sfT ../libgit2-$(LIBGIT2_VERSION)/build/libgit2.a lib/libgit2.a
	touch lib/libgit2.a

libgit2: include/git2.h lib/libgit2.a
