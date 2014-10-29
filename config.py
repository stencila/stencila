#!/usr/bin/env python
#
# Python module for accessing configuration information to assist `Makefile`.
# This module acts as a helper to `Makefile` where Python is easier to write than
# bash/make.
# Some of these fuction could eventually be rewritten in bash or using GNU make functions
# and put back into `Makefile`
import sys
from subprocess import Popen, PIPE

#A function to return output of shell commands
def shell(command):
    return Popen(command, shell=True,stdout=PIPE, stderr=PIPE).communicate()[0].strip()

# Get Stencila version number
def version():
	# Get the long Git description
	# Uses "--long" so that git always output the long format (the tag, the number of commits and 
	#   the abbreviated commit name) even when it matches a tag.
	# Uses "--tags" so that lightweight tags (i.e. tags which are not annotated) are 
	#   shown in output.
	# Uses "--dirty" to indicate uncommitted changes
	long = shell('git describe --long --tags --dirty')
	# Extract into parts
	parts = long.split('-')
	tag = parts[0]
	commits = parts[1]
	commit = parts[2]
	dirty = True if 'dirty' in long else False
	# Create version number
	version = tag
	# If any extra commits or any uncommited changes
	# then add a '+' suffix to indicate a local development
	# branch. We do not add commit to the version number so that completely
	# new builds are not triggered on each commit (only on new tag numbers or
	# changes after them).
	version += '+'
	return version

# Get the operating system  e.g. linux
def os():
	os = shell('uname -o')
	if os=='GNU/Linux': os = 'linux'
	else: os = os.lower()
	return os

# Get the machine architecture e.g i386, x86_64
def arch():
	return shell('uname -m')

# Get the Python major.minor version
def py_version():
	return '%s.%s'%(sys.version_info[0],sys.version_info[1])

# Run function with name of first call argument
if len(sys.argv)>1:	print(locals()[sys.argv[1]]())