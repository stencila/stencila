'''
Setup script for Stencila Python package.

Usage:
    python setup.py build
    sudo python setup.py install
    
For creating binary packages or installers use:
    python setup.py bdist
'''
from distutils.core import setup,Extension
from subprocess import Popen, PIPE

setup(
    #See http://docs.python.org/distutils/apiref.html for a full list of optional arguments
    name = 'stencila',
    version = Popen('git describe', shell=True,stdout=PIPE, stderr=PIPE).communicate()[0],
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