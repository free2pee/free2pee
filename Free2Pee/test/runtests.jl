using JSON3, Downloads, CSV, DataFrames, JSONTables
key = read("key.txt", String);

url = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?location=-33.8670522,151.1957362&radius=500&types=restaurant&name=harbour&key=AIzaSyB9HcAm4LKO54A0IDdLgy0-RxiLhdDvHJU"
j = get_json(url)
p = j.results[1]


url4 = "https://maps.googleapis.com/maps/api/place/details/json?place_id=ChIJN1t_tDeuEmsRUsoyG83frY4&key=$key"
j4 = get_json(url4)

# 186 alewife brook pkwy
lat = 42.39041834486702
long = -71.14001308583926
url5 = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?keyword=chipotle&location=$(lat)%2C$(long)&radius=500&type=restaurant&key=$(key)"
j5 = get_json(url5)
p = j5.results[1]
@test p.name == "Chipotle Mexican Grill"
place_id = p.place_id

# Warning: If you do not specify at least one field with a request, or if you omit the fields parameter from a request, ALL possible fields will be returned, and you will be billed accordingly. This applies only to Place Details requests.

url6 = "https://maps.googleapis.com/maps/api/place/details/json?place_id=$(place_id)&key=$key"
j6 = get_json(url6)
