using Oxygen
using JSON3, Downloads, CSV, DataFrames, JSONTables, PrettyTables
using HTTP, URIs
using Distances

lat = 42.3640906
lon = -71.1018806
()
url = "https://overpass-api.de/api/interpreter"
maps_uri = URI("https://www.google.com/maps/dir/")
h = HTTP.get(test_uri)

example_query = "https://www.google.com/maps/dir/?api=1&origin=<lat, long>&destination=<lat, long>"
u = URI(example_query)
HTTP.queryparams(u)
dump(ex)
get_site(url) = take!(Downloads.download(url, IOBuffer()))
get_json(url) = JSON3.read(get_site(url))

function lat_long_pair(n)
    return (n["lat"], n["lon"])
end

function generate_google_api_url(origin_lat,origin_long, dest_lat, dest_long) 
    URI(maps_uri; query=Dict("api" => 1, "origin" => "$origin_lat,$origin_long", "destination" => "$dest_lat,$dest_long", "dir_action" => "navigate")) 
end 


@get "/{lat}/{lon}" function (req::HTTP.Request, lat, lon)
    lat = parse(Float64, lat)
    lon = parse(Float64, lon)
    # todo make radius query parameter
    query = """
        [out:json];
        node(around:5000, $lat, $lon)["amenity"="toilets"];
        out 10;
    """

    response = HTTP.post(url, nothing, query)
    j = JSON3.read(response.body)
    es = j.elements
    # e = es[1]
    # elat, elon = e.lat, e.lon

    ps = lat_long_pair.(es)
    distances = haversine.(ps, ((lat, lon),))
    sorted_ns = es[sortperm(distances)]
    df = DataFrame(sorted_ns)

    urls = map(x -> generate_google_api_url(lat, lon, x...), ps)
    df.maps_url = urls
    view_df = df[:, [:id, :lat, :lon, :maps_url]]

    io = IOBuffer()
    pretty_table(io, view_df, nosubheader=true, backend=Val(:html))
    HTTP.Response(200, ["Content-Type" => "text/html"]; body=take!(io))

end

serve()
