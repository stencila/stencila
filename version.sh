#!/usr/bin/env sh

# A script to get the Stencila version including build meta data
# 
# The version, based on Semver 2.0.0 (http://semver.org), is added as git tags:
#
#   major.minor.patch
# 
# In addition, # following http://semver.org/#spec-item-10, build meta data is added as the first 7 characters
# of the git SHA *if* there are any commits subsequent to the last tag:
# 
#   major.minor.patch+sha
#   
# This script is intended to be used by Stencila modules (e.g. `node`, `r`) etc to insert the version
# number into configuration files (e.g. `package.json` for `node`, `DESCRIPTION` for `r`). It is
# also available as the `$(VERSION)` make variable via `include ../shared.make`
# 
# Because not all languages support the `+build` syntax for versions, this script accepts an argument which will modify the
# version string to be compatible:

LANG=${1:-none}

# Get stuff
TAG=$(git describe --abbrev=0)
PATCH=$(printf $TAG | sed -n -r 's/([0-9])\.([0-9]+)(\.[0-9]+)?/\3/p')
COMMITS=$(git rev-list $TAG..HEAD --count)
COMMIT=$(git rev-parse HEAD | cut -c1-7)

# Ensure minor is present as per Semver 2.0.0
VERSION=$TAG
if [ "$PATCH" = "" ]; then
  VERSION=$VERSION.0
fi

# Append commit SHA if necessary
if [ "$COMMITS" != "0" ]; then
  if [ "$LANG" = "r" ]; then
  	# In R the build component needs to be another decimal
    VERSION=$VERSION.$((0x$COMMIT))
  else
  	# For everything else the SHA is OK
    VERSION=$VERSION+$COMMIT
  fi
fi

printf $VERSION
