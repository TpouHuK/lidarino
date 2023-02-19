from math import sin, cos
import json
import sys

data = json.load(sys.stdin)

for i, p in enumerate(data):
    if i > 1000:
        break
    yaw = p["yaw"]
    pitch = p["pitch"]
    distance = p["distance"] / 1000;
    x = distance * sin(yaw) * cos(pitch)
    y = distance * sin(pitch)
    z = distance * cos(yaw) * cos(pitch)
    print(x, y, z)
