# Makefile for managing CFF example files
#
# This Makefile provides commands to download, update, and manage
# CITATION.cff example files from popular GitHub repositories.

.PHONY: all download clean list

# Download all example CITATION.cff files
all: download

# Download CITATION.cff files from popular repositories
download:
	@echo "Downloading CITATION.cff examples from GitHub repositories..."
	
	# Pandas - Data analysis library with preferred citation
	curl -s -o pandas.cff https://raw.githubusercontent.com/pandas-dev/pandas/main/CITATION.cff
	
	# PyTorch - Machine learning framework
	curl -s -o pytorch.cff https://raw.githubusercontent.com/pytorch/pytorch/main/CITATION.cff
	
	# TensorFlow - Google's ML system
	curl -s -o tensorflow.cff https://raw.githubusercontent.com/tensorflow/tensorflow/main/CITATION.cff
	
	# eLabFTW - Electronic lab notebook
	curl -s -o elabftw.cff https://raw.githubusercontent.com/elabftw/elabftw/main/CITATION.cff
	
	# LiSC - Literature scanner
	curl -s -o lisc.cff https://raw.githubusercontent.com/lisc-tools/lisc/main/CITATION.cff
	
	# Scientific Inkscape - Figure editing extensions
	curl -s -o scientific-inkscape.cff "https://raw.githubusercontent.com/burghoff/Scientific-Inkscape/main/CITATION.cff"
	
	# Ruby CFF - CFF manipulation library (meta-example)
	curl -s -o ruby-cff.cff https://raw.githubusercontent.com/citation-file-format/ruby-cff/main/CITATION.cff

# List all downloaded files with brief info
list:
	@echo "Available CITATION.cff example files:"
	@for file in *.cff; do \
		if [ -f "$$file" ]; then \
			title=$$(grep "^title:" "$$file" | cut -d'"' -f2 2>/dev/null || echo "No title"); \
			version=$$(grep "^version:" "$$file" | cut -d' ' -f2 2>/dev/null || echo "No version"); \
			echo "  $$file - $$title ($$version)"; \
		fi \
	done

# Clean all downloaded files
clean:
	@echo "Removing all .cff example files..."
	rm -f *.cff
	@echo "Cleaned up example files"

# Update all examples (clean and re-download)
update: clean download
	@echo "Updated all example files"