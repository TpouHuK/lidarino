#!/usr/bin/env python

import asyncio
import websockets

async def hello():
    async with websockets.connect("ws://raspberrypi.local:8000/orientation") as websocket:
        #await websocket.send("Hello world!")
        while True:
            response = await websocket.recv()
            print(response)

asyncio.run(hello())
