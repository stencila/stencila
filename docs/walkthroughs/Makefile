# Get all .smd files in the current directory
SMD_FILES := $(wildcard *.smd)

# Generate list of target .myst files
MYST_FILES := $(SMD_FILES:.smd=.myst)

# Default target to convert all files
all: $(MYST_FILES)

# Pattern rule for converting single files
%.myst: %.smd
	cargo run -p cli convert $< $@ 

# Clean generated files
clean:
	rm -f $(MYST_FILES)
