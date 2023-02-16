using JSON3, Downloads, CSV, DataFrames, JSONTables
using HTTP, URIs
key = read("key.txt", String);

url4 = "https://maps.googleapis.com/maps/api/place/details/json?place_id=ChIJN1t_tDeuEmsRUsoyG83frY4&key=$key";
j4 = get_json(url4);

# 186 alewife brook pkwy
lat = 42.39041834486702
long = -71.14001308583926
url5 = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?keyword=chipotle&location=$(lat)%2C$(long)&radius=500&type=restaurant&key=$(key)";
url5 = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?keyword=food&location=$(lat)%2C$(long)&radius=500&type=restaurant&key=$(key)";
j5 = get_json(url5)
r = j5.results
df = DataFrame(jsontable(r))
chipotle = r[findfirst(x->x.name == "Chipotle Mexican Grill", r)]
@test chipotle.name == "Chipotle Mexican Grill"
place_id = chipotle.place_id
# CSV.write("nearbysearch.csv", df)
# Warning: If you do not specify at least one field with a request, or if you omit the fields parameter from a request, ALL possible fields will be returned, and you will be billed accordingly. This applies only to Place Details requests.
url6 = "https://maps.googleapis.com/maps/api/place/details/json?place_id=$(place_id)&key=$key";
j6 = get_json(url6)
@show j6
