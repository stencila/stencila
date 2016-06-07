'''
Setup script for the Stencila Python package.

To create binary package wheel:
    python setup.py bdist_wheel

The Python Packaging User Guide recommends using setuptools and bdist_wheel (
https://python-packaging-user-guide.readthedocs.org/en/latest/current.html)
'''
import subprocess
from setuptools import setup, Extension
import sys

libraries=[
	'stencila',
	'boost_python',
]
if sys.platform == "win32" :
	libraries += 'z curl ssh2 ssl crypto gdi32 ws2_32 mswsock winhttp crypt32 rpcrt4 ole32'.split()
else:
	libraries += 'curl ssl'.split()

setup(
    name='stencila',
    version=subprocess.check_output('bash ../version.sh', shell=True),

    author='Nokome Bentley',
    author_email='nokome@stenci.la',

    url='http://stenci.la',
    license='GPLv3',

    packages=[
        'stencila'
    ],

    ext_modules=[
        Extension(
            'stencila.extension',
            sources=[
                'stencila/extension.cpp',

                'stencila/component.cpp',
                'stencila/context.cpp',
                'stencila/sheet.cpp',
                'stencila/spread.cpp',
                'stencila/stencil.cpp',
            ],
            include_dirs=[
                '../cpp',
                '../cpp/build/requires/boost',
                '../cpp/build/requires/websocketpp'
            ],
            extra_compile_args=[
                '--std=c++11', '-Wno-unused-local-typedefs',
				# Define BOOST_PYTHON_STATIC_LIB otherwise on Windows
				# dynamic linkage is assumed for the Boost.python library
				'-DBOOST_PYTHON_STATIC_LIB'
            ],
            library_dirs=[
                '../cpp/build/library',
                '../cpp/build/requires/boost/lib'
            ],
            libraries=libraries
        ),
    ],

    scripts=[
        'scripts/stencila-py'
    ]
)
