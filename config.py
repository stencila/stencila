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
	commits = parts[1] if len(parts)>1 else '?'
	commit = parts[2] if len(parts)>2 else '?'
	dirty = True if 'dirty' in long else False
	# Create version number
	version = tag
	if commits!='0': version += '-%s'%commit
	if dirty: version += '-dirty'
	return version

# Get the operating system  e.g. linux, win
def os():
	os = shell('uname -o').lower()
	if os=='gnu/linux': os = 'linux'
	elif os=='msys': os = 'win'
	return os

# Get the machine architecture e.g i386, x86_64
def arch():
	return shell('uname -m')

# Get the Python major.minor version
def py_version():
	return '%s.%s'%(sys.version_info[0],sys.version_info[1])

# Run function with name of first call argument
if len(sys.argv)>1:	print(locals()[sys.argv[1]]())