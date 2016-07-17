include ../shared.make

CPR_VERSION := 1.2.0

build/requires/cpr:
	mkdir -p build/requires
	git clone https://github.com/whoshuu/cpr.git $@
	cd $@ && git checkout $(CPR_VERSION)

CPR_CMAKE_FLAGS := -DBUILD_CPR_TESTS=OFF
ifeq ($(OS), win)
	CPR_CMAKE_FLAGS += -G "MSYS Makefiles"
else
	CPR_CMAKE_FLAGS += -DCMAKE_CXX_FLAGS=-fPIC
endif
build/requires/cpr/build/lib/libcpr.a: build/requires/cpr
	cd build/requires/cpr ;\
		git submodule update --init --recursive ;\
		mkdir -p build ;\
		cd build ;\
		cmake .. $(CPR_CMAKE_FLAGS) ;\
		cmake --build .

cpr: build/requires/cpr/build/lib/libcpr.a
