"""
Module that defines the `Server` class
"""

import asyncio
import json
import signal
import sys

from ..Processor import Processor
from .jsonRpc import Request, Response
from .Logger import Logger

class Server(Logger):
    """
    Base class for all servers.
    """
    
    processor: Processor

    
    def __init__(self, processor: Processor):
        self.processor = processor

    async def start(self) -> None:
        """
        Start this server.

        Starts listening for requests.
        """
        self.log(starting=True)
        await self.open()

    async def open(self) -> None:
        raise NotImplementedError()

    async def stop(self) -> None:
        """
        Stop this server.

        Stops listening for requests.
        """
        await self.close()
        self.log(stopped=True)

    async def close(self) -> None:
        raise NotImplementedError()

    def run(self) -> None:
        loop = asyncio.get_event_loop()
        
        async def run():
            await self.start()
            
            def stop ():
                self.running = False
                asyncio.ensure_future(self.stop())
                loop.remove_signal_handler(signal.SIGINT)
                loop.remove_signal_handler(signal.SIGTERM)
            loop.add_signal_handler(signal.SIGINT, stop)
            loop.add_signal_handler(signal.SIGTERM, stop)
            
            self.running = True
            while self.running:
                await asyncio.sleep(1)

        loop.run_until_complete(run())
        loop.close()
        

    async def receive(self, message: str):
        response = Response()
        try:
            request = self.decode(message)
            
            result = None
            if self.processor:
                if request.method == 'execute':
                    result = self.processor.execute(request.params['thing'])
                    result = result.__dict__
                
            response = Response(id=request.id, result=result)
            self.log(request=request, response=response)
        except Exception as exc:
            response.error = str(exc)
        return self.encode(response)

    def encode(self, response: Response) -> str:
        return json.dumps(response.__dict__)

    def decode(self, message: str) -> Request:
        request = Request()
        request.__dict__.update(json.loads(message))
        return request
