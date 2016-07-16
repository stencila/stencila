JSONCPP_VERSION := 1.7.2

resources/jsoncpp-$(JSONCPP_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/open-source-parsers/jsoncpp/archive/$(JSONCPP_VERSION).tar.gz

build/requires/jsoncpp/dist: resources/jsoncpp-$(JSONCPP_VERSION).tar.gz
	mkdir -p build/requires
	tar xzf $< -C build/requires
	cd build/requires/ ;\
		rm -rf jsoncpp ;\
		mv -f jsoncpp-$(JSONCPP_VERSION) jsoncpp ;\
		cd jsoncpp ;\
			python amalgamate.py ;
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/jsoncpp/dist

requires-jsoncpp: build/requires/jsoncpp/dist
