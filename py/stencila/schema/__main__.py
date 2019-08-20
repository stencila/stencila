import argparse
import logging
from sys import argv, stderr, stdout

from .interpreter import execute_document


def cli_execute():
    execute_document(argv[2:])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Execute stencila.schema from the command line.')

    command = argv[1] if len(argv) > 1 else ''

    if command == 'execute':
        logging.basicConfig(stream=stdout, level=logging.DEBUG)
        cli_execute()
    else:
        stderr.write('Unknown command "{}"\n'.format(command))
