"""
__main__.py can be executed (once this library is installed) like this:

python3 -m stencila.schema execute <inputfile> <outputfile> [parameters]

See READEME.md for more information.

Warning: `eval` and `exec` are used to run code in the document. Don't execute documents that you haven't verified
yourself.
"""

import logging
from sys import argv, stderr, stdout

from .interpreter import execute_from_cli


def cli_execute():
    """Execute an executable document, delegating to the execute_from_cli function."""
    execute_from_cli(argv[2:])


def cli_compile():
    """Compile an executable document by delegating to the execute_from_cli function with the `compile_only` flag."""
    execute_from_cli(argv[2:], True)


def main():
    """The main entry point to this module, read the first CLI arg and call out to the corresponding function."""
    command = argv[1] if len(argv) > 1 else ''

    if command == 'execute':
        logging.basicConfig(stream=stdout, level=logging.DEBUG)
        cli_execute()
    elif command == 'compile':
        logging.basicConfig(stream=stdout, level=logging.DEBUG)
        cli_compile()
    else:
        stderr.write('Unknown command "{}"\n'.format(command))


if __name__ == '__main__':
    main()
