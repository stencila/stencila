from typing import List
import getpass
import os
import stat
import tempfile

def get_tempdir() -> str:
    return os.path.join(tempfile.gettempdir(), 'stencila', getpass.getuser())

def create_tempdir() -> str:
    tempdir = get_tempdir()
    if not os.path.exists(tempdir):
        os.makedirs(tempdir, mode=0o700)
    return tempdir

def get_tempfile(name: str) -> str:
    return os.path.join(create_tempdir(), name)

def list_tempfiles() -> List[str]:
    return os.listdir(get_tempdir())

def create_tempfile(name: str, content: str = None) -> str:
    path = get_tempfile(name)
    
    # Write content to a secure file only readable by current user
    # Based on https://stackoverflow.com/a/15015748/4625911

    # Remove any existing file with potentially elevated mode
    if os.path.isfile(path):
        os.remove(path)

    # Create a file handle
    mode = 0o600
    umask = 0o777 ^ mode  # Prevents always downgrading umask to 0.
    umask_original = os.umask(umask)
    try:
        fd = os.open(path, os.O_WRONLY | os.O_CREAT, mode)
    finally:
        os.umask(umask_original)

    if content:
        # Open file fd and write to file
        with os.fdopen(fd, 'w') as file:
            file.write(content)

    return path

def delete_tempfile(name: str):
    path = get_tempfile(name)
    if os.path.exists(path):
        os.remove(path)
