# See 
# 	https://wiki.allegro.cc/index.php?title=Building_with_msys2
# 	https://ghc.haskell.org/trac/ghc/wiki/Building/Preparation/Windows/MSYS2

pacman -Sy
pacman -S --needed filesystem msys2-runtime bash libreadline libiconv libarchive libgpgme libcurl pacman ncurses libintl

pacman -S base-devel unzip git make mingw-w64-x86_64-cmake
pacman -S mingw-w64-x86_64-toolchain
pacman -S mingw-w64-x86_64-python2 mingw-w64-x86_64-python3
