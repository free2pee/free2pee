cd(@__DIR__)
using Pkg
Pkg.activate("..")
Pkg.instantiate()
using Free2Pee
using JSON3, Downloads, CSV, DataFrames, JSONTables
using HTTP, URIs, Test

key = read(joinpath(@__DIR__, "../../key.txt"), String);

# somewhere close to MIT
lat_lon = [
    "kendall" => (42.36261269868649, -71.08557312482398),
 "central" => (42.36245846895235, -71.10484457066728),
"harvard" => (42.37343282907278, -71.11880134391764),
"porter" => (42.388526745545576, -71.11911680928606),
"davis" => (42.39684669768773, -71.12170552286885),
"alewife" => (42.39590419344493, -71.14152717257815)
]
locations = Dict(lat_lon)

all_res = []
for (stop, (lat, long)) in locations
    url = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?keyword=food&location=$(lat)%2C$(long)&radius=1600&type=restaurant&key=$(key)";
    j = get_json(url)
    results = j.results
    append!(all_res, results)

    while haskey(j,:next_page_token) && j.next_page_token != nothing
        sleep(3)
        url = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?pagetoken=$(j.next_page_token)&keyword=food&location=$(lat)%2C$(long)&radius=1600&type=restaurant&key=$(key)";
        j = get_json(url)
        results = j.results
        append!(all_res, results)
    end
end

all_res = map(identity, all_res)
df = DataFrame(jsontable(JSON3.read(JSON3.write(all_res))))
@test nrow(df) == 360

for pid in df.place_id
    url_pid = "https://maps.googleapis.com/maps/api/place/details/json?place_id=$(pid)&key=$key";
    j_pid = get_json(url_pid)
    write(Free2Pee.data(replace(j_pid.result.name*".json", " "=>"_")), JSON3.write(j_pid))
end