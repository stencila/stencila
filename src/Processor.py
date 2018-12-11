"""
Module that defines the `Processor` class.
"""

import json
from typing import Any, Dict, Union, List, Optional

from .types.Thing import Thing
from .types.utils import cast, hydrate, dehydrate

class Processor:
    """
    The base class for document processors.

    Derived classes may override any of the methods in this class.
    The methods in this class are mostly "no ops" ie. they do not modify the
    `Thing`. They merely serve as an example of how to implement these
    methods in derived classes.
    """
    
    async def hello(self, version: str) -> object:
        return {}
    
    async def goodbye(self) -> None:
        pass

    async def import_(self, thing: Union[str, dict, Thing],
                format: str = 'application/json', type: Optional[str] = None) -> Thing:
        """
        Import a `Thing`.

        This method should generally be called using `super().import_(thing, format, type)` from derived classes.
        """
        if isinstance(thing, Thing):
            return cast(thing, type)
        if isinstance(thing, dict):
            return hydrate(thing, type)
        if isinstance(thing, str):
            if format == 'application/json':
                return hydrate(json.loads(thing), type)
            raise RuntimeError(f'Unhandled import format: {format}')
        raise RuntimeError(f'Unhandled import type: {thing}')

    async def export(self, thing: Thing,
               format: str = 'application/json', type: Optional[str] = None) -> str:
        """
        Export a `Thing`.
        """
        if format == 'application/json':
            return json.dumps(dehydrate(thing), separators=(",", ":"))
        raise RuntimeError(f'Unhandled export format: {format}')


    async def compile(self, thing: Union[str, dict, Thing],
                format: str = 'application/json', type: Optional[str] = None) -> Thing:
        """
        Compile a `Thing`.
        """
        return await self.import_(thing, format, type)

    async def build(self, thing: Union[str, dict, Thing],
              format: str = 'application/json', type: Optional[str] = None) -> Thing:
        """
        Build a `Thing`.
        """
        return await self.compile(thing, format, type)

    async def execute(self, thing: Union[str, dict, Thing],
                format: str = 'application/json', type: Optional[str] = None) -> Thing:
        """
        Execute a `Thing`.
        """
        return await self.build(thing, format, type)
