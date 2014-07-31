# A module for defining functions used during builds
# by both `wscript` and `setup.py`. Putting them here, in a
# separate module allows them to be imported into both scripts.
# 
# Some of these functions could probably make better use of the 
# sysconfig module:
# 
# https://docs.python.org/2.7/library/sysconfig.html
# https://docs.python.org/3.4/library/sysconfig.html

import sys

# Get the Python major.minor version
def py_version():
	return '%s.%s'%(sys.version_info[0],sys.version_info[1])

# Get the Python include directory for system/version combination
def py_include(system,version):
	if system=='linux': return '/usr/include/python%s/'%version
	else: raise NotImplementedError("Sorry, Python builds not yet implemented for operating system <%s>"%system)

# Get the Python extension module file name for system/version combination
def py_extension_name(system,version):
	if(system=='linux'): ext = 'so'
	else: raise NotImplementedError("Sorry, Python builds not yet implemented for operating system <%s>"%system)

	if version<'3.2': return 'extension.%s'%ext
	else: return 'extension.cpython-32mu.%s'%ext
