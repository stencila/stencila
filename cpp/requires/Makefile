all: include lib boost libgit2 pugixml

# Specify that symlinks time should be checked, not the target file time.
# If this option is not specified then symlinks get remade
# Note also in rules below that the rules for creating symlinks should NOT depend upon
# the 'include' and 'lib' rules : if they do then you get cyclic updating because the symlinks
# change the times of those directories
MAKEFLAGS += --check-symlink-times

# Create include and lib directories 
# Where soft links to C++ files and compiled libraries go

include: 
	mkdir -p include

lib:
	mkdir -p lib


include boost.make
include libgit2.make
include pugixml.make
