# Stencila Makefile for [rapidjson](https://code.google.com/p/rapidjson/)

RAPIDJSON_VERSION := 0.11

rapidjson-$(RAPIDJSON_VERSION).zip:
	wget http://rapidjson.googlecode.com/files/rapidjson-$(RAPIDJSON_VERSION).zip

rapidjson-$(RAPIDJSON_VERSION): rapidjson-$(RAPIDJSON_VERSION).zip
	unzip -qo rapidjson-$(RAPIDJSON_VERSION).zip
	mv rapidjson rapidjson-$(RAPIDJSON_VERSION)

include/rapidjson: rapidjson-$(RAPIDJSON_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson include/rapidjson
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/rapidjson rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson
	touch include/rapidjson
endif
	
rapidjson: include include/rapidjson
