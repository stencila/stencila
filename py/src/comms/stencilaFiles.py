import getpass
import os
import platform
import tempfile

def get_homedir() -> str:
    """
    Get the current user's Stencila directory.

    This is the directory that Stencila configuration settings, such as the
    installed Stencila processors get stored.

    :returns: A filesystem path
    """

    osn = platform.system().lower()
    if osn == 'darwin':
        return os.path.join(os.getenv("HOME"), 'Library', 'Application Support', 'Stencila')
    elif osn == 'linux':
        return os.path.join(os.getenv("HOME"), '.stencila')
    elif osn == 'windows':
        return os.path.join(os.getenv("APPDATA"), 'Stencila')
    else:
        return os.path.join(os.getenv("HOME"), 'stencila')

def get_tempdir() -> str:
    """
    Get the Stencila temporary directory for the user.

    :returns: A filesystem path
    """
    return os.path.join(tempfile.gettempdir(), 'stencila', getpass.getuser())

def create_homedir() -> str:
    homedir = get_homedir()
    if not os.path.exists(homedir):
        os.makedirs(homedir)
    return homedir

def create_tempdir() -> str:
    tempdir = get_tempdir()
    if not os.path.exists(tempdir):
        os.makedirs(tempdir, mode=0o700)
    return tempdir

def create_tempfile(name: str, content: str = None) -> str:
    path = os.path.join(create_tempdir(), name)

    if content is not None:
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

        # Open file fd and write to file
        with os.fdopen(fd, 'w') as file:
            file.write(content)

    return path

def delete_tempfile(name: str):
    path = os.path.join(get_tempdir(), name)
    if os.path.exists(path):
        os.remove(path)
