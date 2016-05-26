#!/usr/bin/env sh

# A script to get the Stencila version number
# 
# The version number is based on the last tag plus the number of subsequent commits (if any)
# 
#   major.minor.patch.commits
#   
# The `major.minor.patch` is defined by the tag and `commits` by the number of commits since
# the tag. This format is intended to be compatible with package versioning formats
# for languages (e.g. Python, R)

TAG=$(git describe --abbrev=0)
PATCH=$(printf $TAG | sed -n -r 's/([0-9])\.([0-9]+)(\.[0-9]+)?/\3/p')
COMMITS=$(git rev-list $TAG..HEAD --count)

VERSION=$TAG
if [ "$COMMITS" != "0" ]; then
  if [ "$PATCH" = "" ]; then
  	VERSION=$VERSION.0
  fi
  VERSION=$VERSION.$COMMITS
fi

printf $VERSION
