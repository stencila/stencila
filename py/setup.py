'''
Setup script for Stencila Python package.

For creating binary packages:
    python setup.py bdist_wheel
    
There has been a lot of confusion and contention around Python packaging e.g.
    http://lucumr.pocoo.org/2012/6/22/hate-hate-hate-everywhere/
    http://cournape.wordpress.com/2012/06/23/why-setuptools-does-not-matter-to-me/
    http://blog.ziade.org/2012/09/10/dear-django-help-python-packaging/

The Python Packaging User Guide recommends using setuptools (https://python-packaging-user-guide.readthedocs.org/en/latest/current.html)
and bdist_wheel
'''
import sys
import os
from setuptools import setup, Extension
import glob

from configuration import *

# Get the python version
python_version = py_version()
print("Python version: %s"%python_version)

# Determine correct libraries to use based on Python version
python_lib = 'python%s'%python_version
# On Ubuntu, the ABI
if python_version>='3.2':
    python_lib += 'mu'
boost_python_lib = 'boost_python3' if python_version>='3.0' else 'boost_python'

# Get the Stencila version
stencila_version = os.getenv('STENCILA_VERSION')
print("Stencila version: %s"%stencila_version)

objects = glob.glob('objects/*.o')
print("Object files provided as extra_objects: %s"%objects)

setup(
    # See http://docs.python.org/distutils/apiref.html for a full list of optional arguments
    name = 'stencila',
    version = stencila_version,

    author = 'Nokome Bentley',
    author_email = 'nokome.bentley@stenci.la',

    url = 'http://stenci.la',
    license = 'BSD 3-clause Licence',

    packages = [
        'stencila'
    ],

    # Compile the final extension module here rather than in wscript
    # This ensure that the wheel and extension module that is produced has the correct binary naming
    # convention.
    # Another way around this is described here http://lucumr.pocoo.org/2014/1/27/python-on-wheels/#building-wheels .
    # The method used here appears to produce a wheel layout that is more similar to expected for a 
    # binary distribution.
    ext_modules = [
        Extension(
            'stencila.extension',
            ['stencila/extension.cpp'],
            extra_objects = objects,
            extra_compile_args = ['--std=c++11'],
            library_dirs = [
                '../../cpp/requires/lib'
            ],
            libraries = [
                'boost_filesystem','boost_system','boost_regex',
                'git2','crypto','ssl','rt','z',
                'pugixml',
                'tidy-html5',
                python_lib,
                boost_python_lib,
            ]
        ),
    ],
)
