# pylint: disable=missing-docstring
# pylint: enable=missing-docstring

# pylint: disable=too-few-public-methods


class Thing:
    """The most generic type of item.

    This is base class for all other classes in this type schema.

    https://schema.org/Thing
    """

    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)
