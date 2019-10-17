"""Functions for system-related tasks (currently just registering/deregistering the executor manifest)."""

import json
import logging
import os
import sys
import typing

LOGGER = logging.getLogger(__name__)
LOGGER.addHandler(logging.NullHandler())

MANIFEST = {
    'capabilities': {
        'execute': {
            'type': 'object',
            'required': ['node'],
            'properties': {
                'node': {
                    'type': 'object',
                    'required': ['type', 'programmingLanguage'],
                    'properties': {
                        'type': {
                            'enum': ['CodeChunk', 'CodeExpression']
                        },
                        'programmingLanguage': {
                            'enum': ['python']
                        }
                    }
                }
            }
        }
    },
    'addresses': {
        'stdio': {
            'type': 'stdio',
            'command': 'python3',
            'args': ['-m', 'stencila.schema', 'listen']
        }
    }
}

MANIFEST_FILE_NAME = 'py.json'
EXECUTORS_DIR_NAME = 'executors'


class ManifestManager:
    """Register or deregister a manifest file."""

    @staticmethod
    def get_home_dir() -> typing.Optional[str]:
        """Get the "home" directory of the current user, (or `APPDATA` dir on windows)."""
        os_name = sys.platform.lower()

        if os_name == 'windows':
            return os.getenv('APPDATA')

        return os.getenv('HOME')

    def user_dir(self) -> str:
        """
        Get the current user's Stencila data directory.

        This is the directory that Stencila configuration settings, such as the installed Stencila hosts, and document
        buffers get stored.
        """
        os_name = sys.platform.lower()

        home_dir = self.get_home_dir()

        if not home_dir:
            raise RuntimeError('Could not determine home directory from environment.')

        if os_name == 'darwin':
            return os.path.join(home_dir, 'Library', 'Application Support', 'Stencila')

        if os_name == 'linux':
            return os.path.join(home_dir, '.stencila')

        if os_name == 'windows':
            return os.path.join(home_dir, 'Stencila')

        return os.path.join(home_dir, 'stencila')

    def manifest_dir(self) -> str:
        """Get the directory in which execution registration manifests are stored."""
        return os.path.join(self.user_dir(), EXECUTORS_DIR_NAME)

    def manifest_path(self) -> str:
        """Get the path the the execution registration manifest for this type of executor (Python)."""
        return os.path.join(self.manifest_dir(), MANIFEST_FILE_NAME)

    def register(self) -> None:
        """
        Write the registration manifest (`MANIFEST`) to the manifest path.

        The command path (i.e. path to python) is set to the currently executing python binary, before right. This is to
        provide compatibility with virtual environments. The means that if you re-run the register command with
        different python binaries a different manifest will be written out.
        """
        MANIFEST['addresses']['stdio']['command'] = sys.executable  # type:ignore
        os.makedirs(self.manifest_dir(), exist_ok=True)
        with open(self.manifest_path(), 'w') as manifest_file:
            json.dump(MANIFEST, manifest_file, indent=True)
        LOGGER.info('Manifest saved to \'%s\'', self.manifest_path())

    def deregister(self) -> None:
        """
        Remove the manifest file created with `register`.

        Does not fail if the manifest file does not exist.
        """
        if os.path.exists(self.manifest_path()):
            os.unlink(self.manifest_path())
            LOGGER.info('Deleted manifest at path \'%s\'', self.manifest_path())
        else:
            LOGGER.warning('Not deregistering as file \'%s\' does not exist', self.manifest_path())


def register() -> None:
    """Register the `MANIFEST` as defined by `EXECUTORS_DIR_NAME` and `MANIFEST_FILE_NAME`."""
    ManifestManager().register()


def deregister() -> None:
    """Deregister the `MANIFEST` as defined by `EXECUTORS_DIR_NAME` and `MANIFEST_FILE_NAME`."""
    ManifestManager().deregister()
