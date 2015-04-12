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
import os
from setuptools import setup, Extension

setup(
    # See http://docs.python.org/distutils/apiref.html for a full list of optional arguments
    name = 'stencila',
    version = os.getenv('VERSION'),

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
            ['dummy.cpp'],
            extra_objects = os.getenv('EXTRA_OBJECTS').split(),
            library_dirs = os.getenv('LIBRARY_DIRS').split(),
            libraries = os.getenv('LIBRARIES').split()
        ),
    ],

    # Install CLI
    scripts = [
        'scripts/stencila-py'
    ]
)
