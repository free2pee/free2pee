cd(@__DIR__)
using Pkg
Pkg.activate("..")
Pkg.instantiate()
using Free2Pee
using JSON3, Downloads, CSV, DataFrames, JSONTables
using HTTP, URIs

key = read(joinpath(@__DIR__, "../../key.txt"), String);

# somewhere close to MIT
lat = 42.362125
long = -71.098694
url_kendall = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?keyword=food&location=$(lat)%2C$(long)&radius=1600&type=restaurant&key=$(key)";
j_kendall = get_json(url_kendall)
all_res = []
kendall_results = j_kendall.results
append!(all_res, kendall_results)

while haskey(j_kendall,:next_page_token) && j_kendall.next_page_token != nothing
    println("here")
    url_kendall  = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?pagetoken=$(j_kendall.next_page_token)&keyword=food&location=$(lat)%2C$(long)&radius=1600&type=restaurant&key=$(key)";
    println(url_kendall)
    j_kendall = get_json(url_kendall)
    println(j_kendall)
    println(j_kendall.next_page_token)
    kendall_results = j_kendall.results
    append!(all_res, kendall_results)
end

kendall_results = j_kendall.results
kendall_df = DataFrame(jsontable(kendall_results))
for pid in kendall_df.place_id
    url_pid = "https://maps.googleapis.com/maps/api/place/details/json?place_id=$(pid)&key=$key";
    j_pid = get_json(url_pid)
    write(Free2Pee.data(replace(j_pid.result.name*".json", " "=>"_")), JSON3.write(j_pid))
end

# name, hours, business_status, address, place id, prob of bathroom, uncertainty, bathroom id