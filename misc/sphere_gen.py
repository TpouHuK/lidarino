import matplotlib.pyplot as plt
from math import sin, cos, tau, pi, sqrt, radians
import math
import random

def sphere_point(s, t):
    r = SPHERE_RADIUS
    x = r * cos(s) * sin(t)
    y = r * sin(s) * sin(t)
    z = r * cos(t)
    return (x, y, z)

def circ(r):
    return pi*r*r

points = []
def add_point(point):
    global points
    points.append(point)
   
PHI = pi * (3 - sqrt(5))
PI_2 = pi * 2

def to_spherical(point):
    x, y, z = point
    p = sqrt(x*x + y*y + z*z)
    phi = math.atan2(y, x)
    theta = math.acos(z / sqrt(x*x + y*y + z*z))
    return (p, phi, theta)


def generate_points(radius, index, total, minimum=0.0, maximum=1.0, angleStart=0, angleRange=360):
    y = ( (index / (total - 1)) * (maximum - minimum) + minimum ) * 2.0 - 1.0 #y goes from min- to max+
    theta = PHI * index #golden angel increment
    
    if (angleStart != 0 or angleRange != 360):
        theta = theta % PI_2
        if (theta < 0):
            theta + PI_2
        
        theta = theta * radians(angleRange) / PI_2 + radians(angleStart)   
        
    #radius at y    
    rY = sqrt(1 - y * y)
    x = cos(theta) * rY
    z = sin(theta) * rY
    
    p = (x*radius, y*radius, z*radius)
    add_point(p)
    
    
def reshape_points(points):
    list_x = []
    list_y = []
    list_z = []

    for (x, y, z) in points:
        list_x.append(x)
        list_y.append(y)
        list_z.append(z)

    return (list_x, list_y, list_z)

fig = plt.figure()
#ax = fig.add_subplot(projection='3d')
#ax.set_aspect('equal', adjustable='box')
N = 1000
for i in range(0, N):
    generate_points(1, i, N, 0, 0.75, 0, 360)

print(f"{len(points)} points generated.")
# x, y, z = reshape_points(points)

x_data = []
y_data = []

for point in points:
    _, phy, theta = to_spherical(point)
    x_data.append(phy)
    y_data.append(theta)

min_val = 100000
min_ind = 0
for i, val in enumerate(x_data):
    if val < min_val:
        min_val = val
        min_ind = i


def dist_sq(a, b):
    return (a[0]-b[0])**2 + (a[1]-b[1])**2

path = [min_ind]
while len(path) < len(x_data):
    prev_xy = (x_data[path[-1]], y_data[path[-1]])

    cur_min_dist = 100000000000000
    cur_min_ind = 0

    for i in range(len(x_data)):
        if i in path: continue
        cur_xy = (x_data[i], y_data[i])
        dist = dist_sq(prev_xy, cur_xy)
        if dist < cur_min_dist:
            cur_min_ind = i
            cur_min_dist = dist
    path.append(cur_min_ind)

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
