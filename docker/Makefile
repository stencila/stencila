all: r-build py-build

include ../shared.make

VERSION := $(shell ../version.sh docker)

# When doing `docker push` note that it's necessary to push both version and latest tags:
#   http://container-solutions.com/docker-latest-confusion/
#   https://github.com/docker/docker/issues/7336

# R
r-build:
	cp stencila-session.r ubuntu-14.04-r-3.2
	cp $(shell ls -rt ../r/stencila_*.tar.gz | tail -n 1) ubuntu-14.04-r-3.2/stencila.tar.gz
	docker build --tag stencila/ubuntu-14.04-r-3.2:$(VERSION) ubuntu-14.04-r-3.2
	docker tag stencila/ubuntu-14.04-r-3.2:$(VERSION) stencila/ubuntu-14.04-r-3.2:latest

r-publish:
	docker push stencila/ubuntu-14.04-r-3.2:$(VERSION)
	docker push stencila/ubuntu-14.04-r-3.2:latest
	$(call PUBLISH_NOTIFY,docker,ubuntu-14.04-r-3.2)


# Python
py-build:
	cp stencila-session.py ubuntu-14.04-py-2.7
	cp $(shell ls -rt ../py/dist/*.whl | tail -n 1) ubuntu-14.04-py-2.7/stencila.whl
	docker build --tag stencila/ubuntu-14.04-py-2.7:$(VERSION) ubuntu-14.04-py-2.7
	docker tag stencila/ubuntu-14.04-py-2.7:$(VERSION) stencila/ubuntu-14.04-py-2.7:latest

py-publish:
	docker push stencila/ubuntu-14.04-py-2.7:$(VERSION)
	docker push stencila/ubuntu-14.04-py-2.7:latest
	$(call PUBLISH_NOTIFY,docker,ubuntu-14.04-py-2.7)

# Clean up by removing everything that is not a Docerfile in subdirs
clean:
	rm -rf $$(find ./**/* ! -name Dockerfile)
