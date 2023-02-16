cd(@__DIR__)
using Pkg
Pkg.activate("..")
Pkg.instantiate()

using JSON3, Downloads, CSV, DataFrames, JSONTables
using HTTP, URIs

key = read("key.txt", String);

# somewhere close to MIT
lat = 42.362125
long = -71.098694
url5 = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?keyword=food&location=$(lat)%2C$(long)&radius=500&type=restaurant&key=$(key)";
j5 = get_json(url5)
r = j5.results
df = DataFrame(jsontable(r))
chipotle = r[findfirst(x->x.name == "Chipotle Mexican Grill", r)]
@test chipotle.name == "Chipotle Mexican Grill"
place_id = chipotle.place_id
CSV.write("nearbysearch.csv", df)
# Warning: If you do not specify at least one field with a request, or if you omit the fields parameter from a request, ALL possible fields will be returned, and you will be billed accordingly. This applies only to Place Details requests.
url6 = "https://maps.googleapis.com/maps/api/place/details/json?place_id=$(place_id)&key=$key";
j6 = get_json(url6)