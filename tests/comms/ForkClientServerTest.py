import asyncio
import pytest

from stencilaschema.comms.ForkClientServer import ForkClientServer

from helpers.TestProcessor import TestProcessor

@pytest.mark.asyncio
async def test_fork():

    processor = TestProcessor()
    clerver = ForkClientServer(processor)
    await clerver.start()

    if clerver.is_client:
        # This is the parent client process so run
        # tests from here
        assert await clerver.execute({"type": "Thing"}) == {"type": "Thing"}
        await clerver.stop()
    else:
        # This is the child server process so just sleep
        # until the parent client process kills me
        await asyncio.sleep(10)
