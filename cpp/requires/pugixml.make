include ../shared.make

PUGIXML_VERSION := 1.7

resources/pugixml-$(PUGIXML_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/zeux/pugixml/archive/v$(PUGIXML_VERSION).tar.gz

build/requires/pugixml: resources/pugixml-$(PUGIXML_VERSION).tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/pugixml-$(PUGIXML_VERSION) build/requires/pugixml
	touch $@

PUGIXML_CXX_FLAGS := -O2
ifneq ($(OS), win)
	PUGIXML_CXX_FLAGS += -fPIC
endif
build/requires/pugixml/src/libpugixml.a: build/requires/pugixml
	cd $</src ;\
	  $(CXX) $(PUGIXML_CXX_FLAGS) -c pugixml.cpp ;\
	  $(AR) rcs libpugixml.a pugixml.o

pugixml: build/requires/pugixml/src/libpugixml.a
