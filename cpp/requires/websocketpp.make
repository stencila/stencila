# Stencila Makefile for [WebSocket++](https://github.com/zaphoyd/websocketpp)

WEBSOCKETPP_VERSION := 0.3.0-alpha4

websocketpp-$(WEBSOCKETPP_VERSION).zip:
	wget -O websocketpp-$(WEBSOCKETPP_VERSION).zip https://github.com/zaphoyd/websocketpp/archive/$(WEBSOCKETPP_VERSION).zip

websocketpp-$(WEBSOCKETPP_VERSION): websocketpp-$(WEBSOCKETPP_VERSION).zip
	unzip websocketpp-$(WEBSOCKETPP_VERSION).zip
	touch websocketpp-$(WEBSOCKETPP_VERSION)

include/websocketpp: websocketpp-$(WEBSOCKETPP_VERSION)
	ln -sfT ../websocketpp-$(WEBSOCKETPP_VERSION)/websocketpp include/websocketpp

websocketpp: include/websocketpp
