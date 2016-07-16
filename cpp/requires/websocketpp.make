WEBSOCKETPP_VERSION := 0.7.0

resources/websocketpp-$(WEBSOCKETPP_VERSION).tar.gz:
	mkdir -p resources
	wget -q --no-check-certificate -O $@ https://github.com/zaphoyd/websocketpp/archive/$(WEBSOCKETPP_VERSION).tar.gz

build/requires/websocketpp: resources/websocketpp-$(WEBSOCKETPP_VERSION).tar.gz
	mkdir -p build/requires
	rm -rf $@
	tar xzf $< -C build/requires
	mv build/requires/websocketpp-$(WEBSOCKETPP_VERSION) $@
	touch $@

REQUIRES_INC_DIRS += -Ibuild/requires/websocketpp

requires-websocketpp: build/requires/websocketpp
