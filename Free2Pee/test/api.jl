using Oxygen
using JSON3, Downloads, CSV, DataFrames, JSONTables
using HTTP, URIs

get_site(url) = take!(Downloads.download(url, IOBuffer()))
get_json(url) = JSON3.read(get_site(url))

"actually place is result to search"
place_to_latlong(placej)= (placej.geometry.location.lat, placej.geometry.location.lng)
get_geo(x) = x.geometry
get_loc(x) = get_geo(x).location
@get "/bathroom/{lat}/{long}" function(req::HTTP.Request, lat, long)
    lat = parse(Float64, lat)
    long = parse(Float64, long)
    url5 = "https://maps.googleapis.com/maps/api/place/nearbysearch/json?keyword=food&location=$(lat)%2C$(long)&radius=500&type=restaurant&key=$(key)";
    j5 = get_json(url5)
    r = j5.results

    nts = []
    for r in j5.results
        loc = get_loc(r)
        lat = loc.lat
        long = loc.lng
        nt = (;place_id=r.place_id, name=r.name, lat=lat, long=long)
        push!(nts, nt)
    end
    view = DataFrame(nts)
    df = DataFrame(jsontable(r))
    map(place_to_latlong)
    view = df[:name]
    geos = df.geometry
    geos = map(place_to_latlong, r)
    map(x->x.geometry.location.lat, r)
    ids = df.place_id
    Dict()
    return "hello world!"
end

serve()