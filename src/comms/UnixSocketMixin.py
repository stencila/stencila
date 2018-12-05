class UnixSocketMixin:

    path: str

    def __init__(self, path):
        self.path = path

    @property
    def url(self):
        return f'unix://{self.path}'
