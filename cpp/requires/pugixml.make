# Stencila Makefile for [pugixml](http://pugixml.org/)

include $(dir $(lastword $(MAKEFILE_LIST)))/../../variables.make

PUGIXML_VERSION = 1.2

pugixml-$(PUGIXML_VERSION).tar.gz:
	wget http://pugixml.googlecode.com/files/pugixml-$(PUGIXML_VERSION).tar.gz
	
pugixml-$(PUGIXML_VERSION): pugixml-$(PUGIXML_VERSION).tar.gz
	mkdir -p pugixml-$(PUGIXML_VERSION)
	cd pugixml-$(PUGIXML_VERSION) && tar xzf ../pugixml-$(PUGIXML_VERSION).tar.gz

pugixml-$(PUGIXML_VERSION)/src/libpugixml.a: pugixml-$(PUGIXML_VERSION)
	cd pugixml-$(PUGIXML_VERSION)/src; \
	$(CXX) -O3 -fPIC -c pugixml.cpp; \
	$(AR) rcs libpugixml.a pugixml.o

include/pugixml.hpp: pugixml-$(PUGIXML_VERSION)
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/pugixml.hpp include/pugixml.hpp
	touch include/pugixml.hpp

include/pugiconfig.hpp: pugixml-$(PUGIXML_VERSION)
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/pugiconfig.hpp include/pugiconfig.hpp
	touch include/pugiconfig.hpp

lib/libpugixml.a: pugixml-$(PUGIXML_VERSION)/src/libpugixml.a
	ln -sfT ../pugixml-$(PUGIXML_VERSION)/src/libpugixml.a lib/libpugixml.a

pugixml: include lib include/pugixml.hpp include/pugiconfig.hpp lib/libpugixml.a
