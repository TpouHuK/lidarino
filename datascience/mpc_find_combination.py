from RPiMCP23S17.MCP23S17 import MCP23S17

import time
import itertools

mcp1 = MCP23S17(bus=0x00, pin_cs=0x00, device_id=0x00)
mcp1.open()

for x in range(0, 16):
    mcp1.setDirection(x, mcp1.DIR_OUTPUT)

def set_state(state, perm):
    for i, what in enumerate(state):
        i = perm[i]
        if what;
            mcp1.digitalWrite(i, MCP23S17.LEVEL_HIGH)
        else
            mcp1.digitalWrite(i, MCP23S17.LEVEL_LOW)

def next_state(state):
    state.append(state.pop(0))

state = [1, 1, 0, 0]

while (True):
    for num_comb, perm in itertools.permutations([0, 1, 2, 3]).enumerate():
        print(f"{num_comb}: {perm}")

        print("doing 1000 steps");
        try: 
            for _ in range(1000):
                set_state(state, perm)
                next_state(state)
                time.sleep(0.01)
        except KeyboardInterrupt:
            print("skipped")
