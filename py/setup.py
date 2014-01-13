'''
Setup script for Stencila Python package.

Usage example:
    python setup.py build
    sudo python setup.py install
    
For creating binary packages or installers use:
    python setup.py bdist_egg
    
There has been a lot of confusion and contention around Python packaging e.g.
    http://lucumr.pocoo.org/2012/6/22/hate-hate-hate-everywhere/
    http://cournape.wordpress.com/2012/06/23/why-setuptools-does-not-matter-to-me/
    http://blog.ziade.org/2012/09/10/dear-django-help-python-packaging/
The Python Packaging User Guide recommends using setuptools (https://python-packaging-user-guide.readthedocs.org/en/latest/current.html)
'''
from setuptools import setup,Extension

from subprocess import Popen, PIPE

# Obtain variables from the local Makefile
# A function to return output of shell commands
def shell(command):
    return Popen(command, shell=True,stdout=PIPE, stderr=PIPE).communicate()[0].strip()
# Get Makefile variables. --quiet is needed to prevent make from echoing which directory it is in
version = shell('make --no-print-directory --quiet version')
cpp_flags = shell('make --no-print-directory --quiet cpp_flags').split()
cpp_incl_dirs = shell('make --no-print-directory --quiet cpp_include_dirs').replace('-I','').split()
cpp_lib_dirs = shell('make --no-print-directory --quiet cpp_library_dirs').replace('-L','').split()
cpp_libs = shell('make --no-print-directory --quiet cpp_libs').replace('-l','').split()

setup(
    # See http://docs.python.org/distutils/apiref.html for a full list of optional arguments
    name = 'stencila',
    version = version,
    keywords = [],
    author = 'Nokome Bentley',
    author_email = 'nokome.bentley@stenci.la',
    url = 'http://stenci.la',
    license = 'BSD 3-clause Licence',
    packages = [
        'stencila'
    ],
    ext_modules = [
        Extension(
            'stencila.extension',
            [
                'stencila/stencila.cpp'
            ],
            extra_compile_args = cpp_flags,
            include_dirs = cpp_incl_dirs,
            library_dirs = cpp_lib_dirs,
            libraries = ['boost_python']+cpp_libs
        )
    ]
)
