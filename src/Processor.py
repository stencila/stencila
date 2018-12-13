"""
Module that defines the `Processor` class.
"""

import json
from typing import Any, Dict, Union, List, Optional, Type

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
    
    def __init__(self, client_types: List[Type['Client']] = None, server_types: List[Type['Server']] = None):
        self.client_types = client_types
        self.server_types = server_types

        self.clients = {}
        self.servers = {}
        self.logger = None


    async def start(self):
        for server_type in self.server_types:
            server = server_type(self)
            await server.start()
            self.servers[server.url] = server

    async def stop(self):
        for url in list(self.servers):
            server = self.servers[url]
            await server.stop()
            del self.servers[url]

    async def connect(self, url):
        """
        Connect to a peer processor.
        """
        if url not in self.clients:
            success = False
            for client_type in self.client_types:
                if client_type.connectable(url):
                    client = client_type(url)
                    await client.start()
                    self.clients[url] = client
                    success = True
                    break
            if not success:
                raise RuntimeError(f'No client types able to connect to {url}')

    async def disconnect(self, url: str = None):
        if url is None:
            for url in list(self.clients):
                await self.disconnect(url)
        if url in self.clients:
            await self.clients[url].stop()
            del self.clients[url]

    async def discover(self):
        for client_type in self.client_types:
            for client in await client_type.discover():
                # Ensure that not connecting to one of my own servers
                # or a server that I'm already connected to
                if client.url not in self.servers and client.url not in self.clients:
                    self.clients[client.url] = client
    
    async def hello(self, version: str) -> Dict:
        return {}
    
    async def goodbye(self) -> Dict:
        return {}

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
