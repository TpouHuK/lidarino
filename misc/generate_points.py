from math import sin, cos, tau, pi, sqrt, radians
import matplotlib.pyplot as plt
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
ax = fig.add_subplot(projection='3d')
#ax.set_aspect('equal', adjustable='box')
N = 100
for i in range(1, N):
    generate_points(1, i, N, 0, 0.75, 0, 360)


print(f"{len(points)} points generated.")
x, y, z = reshape_points(points)
ax.scatter(x, y, z)

ax.set_xlabel('X Label')
ax.set_ylabel('Y Label')
ax.set_zlabel('Z Label')

plt.axis('equal')
plt.show()

