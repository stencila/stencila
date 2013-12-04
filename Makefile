.PHONY: bin cpp py r sqlite guide

all: bin cpp py r sqlite guide

bin:
	$(MAKE) -C bin all

cpp:
	$(MAKE) -C cpp all
	
py:
	$(MAKE) -C py all
	
r:
	$(MAKE) -C r all
	
sqlite:
	$(MAKE) -C sqlite all
	
guide:
	$(MAKE) -C guide all

