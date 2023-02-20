using Oxygen
using JSON3, Downloads, CSV, DataFrames, JSONTables, PrettyTables
using HTTP, URIs
using Distances

# lat = 42.3640906
# lon = -71.1018806
const LAT = 42.35810736568732
const LON = -71.10592447014132
url = "https://overpass-api.de/api/interpreter"
lat = LAT
lon = LON
# we actually want to use the query that this taginfo project represents
# https://raw.githubusercontent.com/pietervdvn/MapComplete/develop/Docs/TagInfo/mapcomplete_toilets.json
query = """
[out:json];
(
node(around:1000, $lat, $lon)["amenity"="toilets"];
node(around:1000, $lat, $lon)["toilets"="yes"];
);
out;
"""
query = """
[out:json];
node(around:1000, 42.35810736568732, -71.10592447014132);
out;
"""
function gen_query(lat, lon)
query = """
[out:json];
(
    node(around:1000, $lat, $lon)["amenity"="toilets"];
    node(around:1000, $lat, $lon)["toilets:position"="seated"];
    node(around:1000, $lat, $lon)["toilets:position"="urinal"];
    node(around:1000, $lat, $lon)["toilets:position"="squat"];
    node(around:1000, $lat, $lon)["toilets:position"="seated;urinal"];
    node(around:1000, $lat, $lon)["changing_table"="yes"];
    node(around:1000, $lat, $lon)["changing_table"="no"];
    node(around:1000, $lat, $lon)["changing_table:location"];
    node(around:1000, $lat, $lon)["changing_table:location"="female_toilet"];
    node(around:1000, $lat, $lon)["changing_table:location"="male_toilet"];
    node(around:1000, $lat, $lon)["changing_table:location"="wheelchair_toilet"];
    node(around:1000, $lat, $lon)["changing_table:location"="dedicated_room"];
    node(around:1000, $lat, $lon)["toilets:handwashing"="yes"];
    node(around:1000, $lat, $lon)["toilets:handwashing"="no"];
    node(around:1000, $lat, $lon)["toilets:paper_supplied"="yes"];
    node(around:1000, $lat, $lon)["toilets:paper_supplied"="no"];
    node(around:1000, $lat, $lon)["toilets"="yes"];
    node(around:1000, $lat, $lon)["toilets:access"];
    node(around:1000, $lat, $lon)["toilets:fee"="yes"];
    node(around:1000, $lat, $lon)["toilets:fee"="no"];
    node(around:1000, $lat, $lon)["toilets:charge"];
    node(around:1000, $lat, $lon)["toilets:wheelchair"="yes"];
    node(around:1000, $lat, $lon)["toilets:wheelchair"="no"];
    node(around:1000, $lat, $lon)["toilets:wheelchair"="designated"];
    node(around:1000, $lat, $lon)["toilets:door:width"];
    node(around:1000, $lat, $lon)["toilets:position"="seated"];
    node(around:1000, $lat, $lon)["toilets:position"="urinal"];
    node(around:1000, $lat, $lon)["toilets:position"="squat"];
    node(around:1000, $lat, $lon)["toilets:position"="seated;urinal"];
    node(around:1000, $lat, $lon)["changing_table"="yes"];
    node(around:1000, $lat, $lon)["changing_table"="no"];
    node(around:1000, $lat, $lon)["changing_table:location"];
    node(around:1000, $lat, $lon)["changing_table:location"="female_toilet"];
    node(around:1000, $lat, $lon)["changing_table:location"="male_toilet"];
    node(around:1000, $lat, $lon)["changing_table:location"="wheelchair_toilet"];
    node(around:1000, $lat, $lon)["changing_table:location"="dedicated_room"];
    node(around:1000, $lat, $lon)["toilets:handwashing"="yes"];
    node(around:1000, $lat, $lon)["toilets:handwashing"="no"];
    node(around:1000, $lat, $lon)["toilets:paper_supplied"="yes"];
    node(around:1000, $lat, $lon)["toilets:paper_supplied"="no"];
    node(around:1000, $lat, $lon)["toilets:description"];
);
out;
"""
end

response = HTTP.post(url, nothing, query)
j = JSON3.read(response.body)
es = j.elements
@info length(es)
fn = _data("mapcomplete_toilets.json")
jt = JSON3.read(read(fn))
ts = jt.tags

df = to_df(lat, lon)

println.("node(around:1000, $lat, $lon)" .* node_string.(ts))
