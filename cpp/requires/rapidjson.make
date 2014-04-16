# Stencila Makefile for [rapidjson](https://code.google.com/p/rapidjson/)
# 
# There are several forks of rapidjson on Github
# At the time of writing the ones that appeared to be most worthwhile watching were:
# 
# 	- https://github.com/pah/rapidjson
# 	- https://github.com/miloyip/rapidjson/issues/1
# 
# We use a patch from https://github.com/scanlime/rapidjson/commit/0c69df5ac098640018d9232ae71ed1036c692187
# that allows for copying of Documents [rapidjson prevents copying of documents](http://stackoverflow.com/questions/22707814/perform-a-copy-of-document-object-of-rapidjson)

RAPIDJSON_VERSION := 0.11

rapidjson-$(RAPIDJSON_VERSION).zip:
	wget http://rapidjson.googlecode.com/files/rapidjson-$(RAPIDJSON_VERSION).zip

rapidjson-$(RAPIDJSON_VERSION): rapidjson-$(RAPIDJSON_VERSION).zip
	unzip -qo rapidjson-$(RAPIDJSON_VERSION).zip
	mv rapidjson rapidjson-$(RAPIDJSON_VERSION)
	cd rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson; patch -p1 -i ../../../rapidjson-scanlime-0c69df5ac0.patch

include/rapidjson: rapidjson-$(RAPIDJSON_VERSION)
ifeq ($(STENCILA_PLATFORM), linux)
	ln -sfT ../rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson include/rapidjson
endif
ifeq ($(STENCILA_PLATFORM), msys)
	junction include/rapidjson rapidjson-$(RAPIDJSON_VERSION)/include/rapidjson
	touch include/rapidjson
endif
	
rapidjson: include include/rapidjson
