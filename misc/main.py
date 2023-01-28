import matplotlib.pyplot as plt
import json
import math

filename = "vertical_2000.json"

data = json.load(open(filename))
print(data)

x_data = []
y_data = []

for line in data:
    angle = math.radians(90.0 + line["pitch"] / 2000.0 * 90.0)
    distance = line["distance_mm"] / 1000
    x, y = distance * math.sin(angle), distance * math.cos(angle)
    x_data.append(x)
    y_data.append(y)

plt.scatter(x_data, y_data)
plt.show()
