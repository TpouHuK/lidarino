from sys import stdin
import matplotlib.pyplot as plt
import matplotlib as mpl
import json
import random
import sys

def reshape_points(points):
    list_x = []
    list_y = []
    list_z = []

    for (x, y, z) in points:
        list_x.append(x)
        list_y.append(y)
        list_z.append(z)

    return (list_x, list_y, list_z)

points = []
colors = []

data = json.load(open("./pillow_down_1000.json"))
for i, p in enumerate(data):
    #if p["quality"] > 1000:
        #continue
    points.append((p["x"], p["y"], p["z"]))
    colors.append(p["quality"])


data = json.load(open("combined_floor_1000_feb20.json"))
for i, p in enumerate(data):
    if p["quality"] > 1000:
        continue
    points.append((p["x"], p["y"], p["z"]))
    colors.append(p["quality"])

data = json.load(open("./pillows_1000.json"))
for i, p in enumerate(data):
    if p["quality"] > 1000:
        continue
    points.append((p["x"], p["y"], p["z"]))
    colors.append(p["quality"])

data = json.load(open("./3k_wall.json"))
for i, p in enumerate(data):
    if p["quality"] > 1000:
        continue
    points.append((p["x"], p["y"], p["z"]))
    colors.append(p["quality"])

x, y, z = reshape_points(points)

fig = plt.figure()
ax = fig.add_subplot(projection='3d')
ax.set_aspect('equal', adjustable='box')
ax.scatter(x, y, z, c=colors, cmap = mpl.colormaps["plasma"])
ax.set_xlabel('X Label')
ax.set_ylabel('Y Label')
ax.set_zlabel('Z Label')

plt.axis('equal')
plt.show()

