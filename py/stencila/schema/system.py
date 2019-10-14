import json
import logging
import os
import sys

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
    @staticmethod
    def user_dir():
        """
        Get the current user's Stencila data directory.
        This is the directory that Stencila configuration settings, such as the
        installed Stencila hosts, and document buffers get stored.
        :returns: A filesystem path
        """

        osn = sys.platform.lower()
        if osn == 'darwin':
            return os.path.join(os.getenv("HOME"), 'Library', 'Application Support', 'Stencila')
        elif osn == 'linux':
            return os.path.join(os.getenv("HOME"), '.stencila')
        elif osn == 'windows':
            return os.path.join(os.getenv("APPDATA"), 'Stencila')
        else:
            return os.path.join(os.getenv("HOME"), 'stencila')

    def manifest_dir(self):
        return os.path.join(self.user_dir(), EXECUTORS_DIR_NAME)

    def manifest_path(self):
        return os.path.join(self.manifest_dir(), MANIFEST_FILE_NAME)

    def register(self):
        MANIFEST['addresses']['stdio']['command'] = sys.executable
        os.makedirs(self.manifest_dir(), exist_ok=True)
        with open(self.manifest_path(), 'w') as f:
            json.dump(MANIFEST, f, indent=True)
        LOGGER.info('Manifest saved to %s', self.manifest_path())

    def deregister(self):
        if os.path.exists(self.manifest_path()):
            os.unlink(self.manifest_path())
            LOGGER.info('Deleted manifest at path %s', self.manifest_path())
        else:
            LOGGER.warning('Not deregistering as path %s does not exist', self.manifest_path())


def register():
    mm = ManifestManager()
    mm.register()


def deregister():
    mm = ManifestManager()
    mm.deregister()
