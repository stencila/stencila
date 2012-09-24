'''
Setup script for Stencila Python package.

Usage:
    python setup.py build
    sudo python setup.py install
    
For creating binary packages or installers use:
    python setup.py bdist_egg
    
There is a lot of confusion and contention around Python packaging at the moment e.g.
    http://lucumr.pocoo.org/2012/6/22/hate-hate-hate-everywhere/
    http://cournape.wordpress.com/2012/06/23/why-setuptools-does-not-matter-to-me/
    http://blog.ziade.org/2012/09/10/dear-django-help-python-packaging/
The only reason for using setuptools here is that it gets the job done with the bdist_egg command
and at the time of writing distutils2 documentation seemed to be completley out of sync with the code.
'''
from setuptools import setup,Extension

from subprocess import Popen, PIPE

setup(
    #See http://docs.python.org/distutils/apiref.html for a full list of optional arguments
    name = 'stencila',
    version = Popen('git describe', shell=True,stdout=PIPE, stderr=PIPE).communicate()[0].strip(),
    keywords = [],
    author = 'Nokome Bentley',
    author_email = 'nokome.bentley@stenci.la',
    url = 'http://stenci.la',
    license = 'ISC Licence',
    packages = [
        'stencila'
    ],
    ext_modules = [
        Extension(
            'stencila.extension',
            ['stencila/extension.cpp'],
            include_dirs = ['../cpp/','../cpp/requirements/include'],
            extra_compile_args = ['-std=c++0x'],
            library_dirs = ['../cpp/requirements/lib'],
            libraries = ['boost_python','sqlite3','boost_system','boost_filesystem']
        )
    ]
)