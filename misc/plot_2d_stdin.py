import matplotlib.pyplot as plt
from math import sin, cos, tau, pi, sqrt, radians
import 

points = []
for line in stdin:
    point = tuple(map(float, line.split()))
    points.append(point)

fig = plt.figure()
#ax = fig.add_subplot(projection='3d')
#ax.set_aspect('equal', adjustable='box')

x_data = []
y_data = []

for point in points:
    phy, theta = point
    x_data.append(phy)
    y_data.append(theta)

plt.scatter(x_data, y_data)
path_x = [x_data[i] for i in path]
path_y = [y_data[i] for i in path]

plt.plot(path_x, path_y, color="#ff00ff")

plt.axis('equal')
plt.show()

#ax.scatter(x, y, z)

#ax.set_xlabel('X Label')
#ax.set_ylabel('Y Label')
#ax.set_zlabel('Z Label')

#plt.show()

