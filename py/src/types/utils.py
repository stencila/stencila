"""
Module that define utility functions for handling instances.
"""

from typing import Any, Dict, Optional

from .Thing import Thing

def cast(thing: Thing, type: Optional[str] = None) -> Thing:
    # TODO
    return thing

def hydrate(obj: dict, type: Optional[str] = None) -> Thing:
    # TODO use type attribute in obj and check it against type
    return Thing(**obj)

def dehydrate(thing: Thing) -> Dict[str, Any]:
    return vars(thing)
