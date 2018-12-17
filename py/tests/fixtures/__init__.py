import os

def fixture(*path):
    return os.path.join(os.path.join(os.path.dirname(__file__), *path))
