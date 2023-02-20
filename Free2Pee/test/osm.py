import requests
import json

# Set the API endpoint URL
url = 'https://overpass-api.de/api/interpreter'
latitude = 42.3640906
longitude = -71.1018806
# Set the query to retrieve nodes with toilet attributes within 500 meters of the current location
query = f"""
    [out:json];
    node(around:10000, {latitude}, {longitude})["amenity"="toilets"];
    out;
"""

# Send the request to the API
response = requests.post(url, data=query)

# Parse the JSON response
data = json.loads(response.text)
j = json.dumps(data)

with open("sample.json", "w") as outfile:
    outfile.write(j)

# Sort the nodes by distance to the current location
nodes = data['elements']

def dist(nx, ny, nx2, ny2):
    return ((nx - nx2) ** 2 + (ny - ny2) ** 2) ** 0.5

dist(nlat, latitude, nlon, longitude)

# Print the 10 nearest nodes
for i, node in enumerate(nodes):
    nlat, nlon = node['lat'], node['lon']
    d = dist(nlat, latitude, nlon, longitude)
    print(f'{i + 1}. Node {node["id"]} is {d} meters away.')
