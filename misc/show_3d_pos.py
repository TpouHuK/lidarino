#!/bin/python3

import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
from matplotlib.animation import FuncAnimation

import asyncio
import websockets

pitch = 0.0
roll = 0.0
yaw = 0.0

from math import degrees, tau

async def hello():
    global pitch
    global roll
    global yaw
    async with websockets.connect("ws://raspberrypi.local:8000/orientation") as websocket:
        #await websocket.send("Hello world!")
        while True:
            response = await websocket.recv()
            roll, pitch, yaw = map(float, response.split(","))
            print(degrees(roll), degrees(pitch), degrees(yaw))

from threading import Thread
Thread(target = lambda : asyncio.run(hello()), daemon=True).start()



# Define the dimensions of the cuboid
width = 3
height = 5
depth = 2

# Define the pitch, yaw, and roll angles in radians
pitch = np.deg2rad(30)
yaw = np.deg2rad(45)
roll = np.deg2rad(60)

# Define the corners of the cuboid
corners = np.array([
    [-width/2, -height/2, -depth/2],
    [ width/2, -height/2, -depth/2],
    [ width/2,  height/2, -depth/2],
    [-width/2,  height/2, -depth/2],
    [-width/2, -height/2,  depth/2],
    [ width/2, -height/2,  depth/2],
    [ width/2,  height/2,  depth/2],
    [-width/2,  height/2,  depth/2]
])

def get_rotation_matrix():
    # Define the rotation matrix
    #rotation_matrix = np.array([
        #[np.cos(yaw)*np.cos(roll), np.cos(yaw)*np.sin(roll)*np.sin(pitch)-np.sin(yaw)*np.cos(pitch), np.cos(yaw)*np.sin(roll)*np.cos(pitch)+np.sin(yaw)*np.sin(pitch)],
        #[np.sin(yaw)*np.cos(roll), np.sin(yaw)*np.sin(roll)*np.sin(pitch)+np.cos(yaw)*np.cos(pitch), np.sin(yaw)*np.sin(roll)*np.cos(pitch)-np.cos(yaw)*np.sin(pitch)],
        #[-np.sin(roll), np.cos(roll)*np.sin(pitch), np.cos(roll)*np.cos(pitch)]
    #])
    theta1 = yaw
    theta2 = pitch
    theta3 = roll

    c1 = np.cos(theta1)
    s1 = np.sin(theta1)
    c2 = np.cos(theta2)
    s2 = np.sin(theta2)
    c3 = np.cos(theta3)
    s3 = np.sin(theta3)

    matrix=np.array([[c1*c2, c1*s2*s3-c3*s1, s1*s3+c1*c3*s2],
                             [c2*s1, c1*c3+s1*s2*s3, c3*s1*s2-c1*s3],
                             [-s2, c2*s3, c2*c3]])
    return matrix

def get_rotated_corners():
    # Rotate the corners
    rotated_corners = np.dot(get_rotation_matrix(), corners.T).T
    return rotated_corners

# Define the vertices and faces of the cuboid
vertices = get_rotated_corners()
faces = np.array([
    [0, 1, 2, 3],
    [1, 5, 6, 2],
    [5, 4, 7, 6],
    [4, 0, 3, 7],
    [0, 4, 5, 1],
    [3, 2, 6, 7]
])

# Define the colors for each face
colors = ['r', 'g', 'b', 'y', 'm', 'c']

# Plot the cuboid
fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')

ax.set_box_aspect((np.ptp(vertices[:,0]), np.ptp(vertices[:,1]), np.ptp(vertices[:,2])))
ax.set_xlim3d([np.min(vertices[:,0])-1, np.max(vertices[:,0])+1])
ax.set_ylim3d([np.min(vertices[:,1])-1, np.max(vertices[:,1])+1])
ax.set_zlim3d([np.min(vertices[:,2])-1, np.max(vertices[:,2])+1])

cuboid = Poly3DCollection(vertices[faces], alpha=0.25)
cuboid.set_facecolor(colors)
cuboid.set_edgecolor("black")

#for i in range(len(faces)):
    #cuboid = Poly3DCollection([vertices[faces[i]]], alpha=1.0)
    #cuboid.set_facecolor(colors[i])
    #cuboid.set_edgecolor('black')
    #ax.add_collection3d(cuboid)
#cuboid.set_facecolor('blue')

ax.add_collection3d(cuboid)
ax.set_xlabel('X')
ax.set_ylabel('Y')
ax.set_zlabel('Z')

def update(frame):
    vertices = get_rotated_corners()
    cuboid.set(verts=vertices[faces]);
    #for i in range(len(faces)):
        #cuboid = Poly3DCollection([vertices[faces[i]]], alpha=1.0)
        #cuboid.set_facecolor(colors[i])
        #cuboid.set_edgecolor('black')
        #ax.add_collection3d(cuboid)

ani = FuncAnimation(fig, update, frames=range(100), interval=1000/60)

plt.show()

