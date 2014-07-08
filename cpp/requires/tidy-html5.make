# Stencila Makefile for [tidy-html5](http://w3c.github.com/tidy-html5/)
# 
# Note that we only "make ../../lib/libtidy.a" and not "make all" because the latter is not required
# Apply patch to Makefile to add -O3 -fPIC options
# Apply patch from pull request #98 to add <main> tag (this is applied using `patch` rather than `git` so that `git` is not required)
# On MSYS also apply patch to prevent linker error associated with "GetFileSizeEx"

include $(dir $(lastword $(MAKEFILE_LIST)))/../../variables.make

tidy-html5.zip:
	wget --no-check-certificate -Otidy-html5.zip https://github.com/w3c/tidy-html5/archive/master.zip

tidy-html5-master: tidy-html5.zip
	unzip -f tidy-html5.zip
	patch tidy-html5-master/build/gmake/Makefile tidy-html5-build-gmake-Makefile.patch
	wget --no-check-certificate -Otidy-html5-pull-98.patch https://github.com/w3c/tidy-html5/pull/98.patch
	cd tidy-html5-master; patch -p1 -i ../tidy-html5-pull-98.patch
ifeq ($(STENCILA_PLATFORM), msys)
	patch tidy-html5-master/src/mappedio.c tidy-html5-src-mappedio.c.patch
endif

tidy-html5-master/lib/libtidy.a:
	cd tidy-html5-master/build/gmake; make ../../lib/libtidy.a

include/tidy-html5: tidy-html5-master
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../tidy-html5-master/include include/tidy-html5
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/tidy-html5 tidy-html5-master/include
	touch include/tidy-html5
endif

lib/libtidy-html5.a: tidy-html5-master/lib/libtidy.a
	ln -sf ../tidy-html5-master/lib/libtidy.a lib/libtidy-html5.a

tidy-html5: include include/tidy-html5 lib lib/libtidy-html5.a
