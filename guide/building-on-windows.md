# Building Stencila on Windows

These instructions are for setting up a [MSYS](http://www.mingw.org/wiki/MSYS) enviroment on Windows so that you can build the Stencila library.
We don't recommend building the library yourself and you don't have to because we have compiled packages available for Windows.
But if you really want to do that, what follows is just a set of recommendations for how to setup a MSYS enviroment.
You can always choose to do things differently; but you might find you encounter more issues than you need to. :)

## Install MSYS

MSYS (Minimal SYStem) allows us to start using a Linux-like command line as soon as possible,

* Open Internet Explorer or another browser
* Go to http://sourceforge.net/projects/mingw/files/Installer/mingw-get-inst/
* Download and run the latest exe file (mingw-get-inst-20120436.exe at time of writing)
* Select "Use pre-packaged repository catalogues"
* Select "I accept the agreement"
* Select defaults for install location and shortcuts location
* When selecting components, deselect all compilers and only choose "MSYS Basic System"

Create a desktop shortcut to the MSYS prompt: 

* Desktop right click > New > Shortcut
* Enter "C:\MinGW\msys\1.0\msys.bat --norxvt"

## Install basic build tools and libraries required

From the MSYS prompt:

    mingw-get update
    mingw-get install msys-wget msys-unzip msys-zip msys-bzip2 msys-libbz2 msys-libopenssl gcc g++
    
## Install junction

MSYS does not support symbolic links on directories. For Vista there is `mklink` but 
for XP we need to use a program called Junction (http://technet.microsoft.com/en-us/sysinternals/bb896768.aspx).
See http://en.wikipedia.org/wiki/Symbolic_link#Microsoft_Windows.
The Stencila Makefiles for MSYS assume junction is installed, so even if you have Vista or above, you will need it.
Download and unzip it into somewhere in the PATH:

    wget http://download.sysinternals.com/files/Junction.zip
    unzip Junction.zip -d /c/WINDOWS
    
## Install mingw-w64

mingw-w64 is allows for the compilation of 64-bit binaries. We install it
within the "C:/MinGW" directory so it sits along side "mingw32" that is
already there from the previous step.

    wget -O mingw-w64-bin_i686-mingw_20111220.zip http://downloads.sourceforge.net/project/mingw-w64/Toolchains%20targetting%20Win64/Automated%20Builds/mingw-w64-bin_i686-mingw_20111220.zip
    unzip mingw-w64-bin_i686-mingw_20111220.zip -d /mingw/mingw64
    
## Install Git for Windows

Git for Windows allows you to `git clone` the Stencila, and other required, repositories

    wget -O Git-1.8.0-preview20121022.exe http://msysgit.googlecode.com/files/Git-1.8.0-preview20121022.exe
    Git-1.8.0-preview20121022.exe
    
* Select "Run Git from the Windows Command Prompt"
* Select "Checkout Windows-style, commit Unix-style line endings"
* Close and reopen the MSYS console to be able to access git

## Install CMake

CMake allows you to make `cpp-netlib` which is one of the requirements for the Stencila C++ library

    wget http://www.cmake.org/files/v2.8/cmake-2.8.10.2-win32-x86.exe
    cmake-2.8.10.2-win32-x86.exe
    
* Choose "Add CMAke to the system PATH for all users"

## Install R

You need R if you are going to build the Stencila R package.

During installation:
    * Select "C:\" as installation directory because having a space in the path causes problems building with Rcpp
    * Select "64-bit Files" as well as the default "Core Files" and "32-bit Files"
    
    wget http://cran.r-project.org/bin/windows/base/R-2.15.2-win.exe
    R-2.15.2-win.exe
    
Add "C:\R\R-2.15.2\bin" to your path

## Install Python

You need Python if you are going to build the Stencila python package.
The installation defaults should be fine.

    wget http://www.python.org/ftp/python/2.7.3/python-2.7.3.msi
    msiexec -i python-2.7.3.msi

Add "C:\Python27" to your path

## Install `setuptools` and `pip`

    wget http://pypi.python.org/packages/2.7/s/setuptools/setuptools-0.6c11.win32-py2.7.exe
    setuptools-0.6c11.win32-py2.7.exe
    
Add "C:\Python27\Scripts" to your path and open up a new MSYS prompt:

    easy_install pip

## Get Stencila repository

Clone a read only version of repo

    git clone git://github.com/stencila/stencila.git

## Make Stencila
    
    cd stencila
    make all
    
Note that when building the Boost library with a different compiler you need to 
specifying the full path in `user-config.jam` otherwise `b2` does not seem to be able to find it. e.g

    echo 'using gcc : : "C:/MinGW/mingw64/bin/x86_64-w64-mingw32-g++"; ' > tools/build/v2/user-config.jam ;


   
