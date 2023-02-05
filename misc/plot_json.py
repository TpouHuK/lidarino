from sys import stdin
import matplotlib.pyplot as plt
import json

def reshape_points(points):
    list_x = []
    list_y = []
    list_z = []

    for (x, y, z) in points:
        list_x.append(x)
        list_y.append(y)
        list_z.append(z)

    return (list_x, list_y, list_z)

data = json.load(open("points.json"))
points = []

for p in data:
    points.append((p["x"], p["y"], p["z"]))

x, y, z = reshape_points(points)

fig = plt.figure()
ax = fig.add_subplot(projection='3d')
ax.set_aspect('equal', adjustable='box')
ax.scatter(x, y, z)
ax.set_xlabel('X Label')
ax.set_ylabel('Y Label')
ax.set_zlabel('Z Label')

plt.axis('equal')
plt.show()

