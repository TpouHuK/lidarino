import requests
import json
points = json.load(open("result_lowlat_points.json"))

new_points = []
for point in points: 
    new_point = {
            "robot": 1,
            "point_x": point["x"],
            "point_y": point["y"],
            "point_z": point["z"],
            }
    new_points.append(new_point)

to_send = { "points_list" : new_points }
endpoint = "http://10.13.202.3:8080/api/v1/subplace/get_2d_points_map?robot_id=1"
r = requests.post(endpoint, data=to_send)
print(r)
