using Oxygen
using JSON3, Downloads, CSV, DataFrames, JSONTables, PrettyTables
using HTTP, URIs
using Distances

# lat = 42.3640906
# lon = -71.1018806
const LAT = 42.35810736568732
const LON = -71.10592447014132
lat = LAT
lon = LON
url = "https://overpass-api.de/api/interpreter"
const MAPS_URI = URI("https://www.google.com/maps/dir/")
# h = HTTP.get(test_uri)
# 4360543295
example_query = "https://www.google.com/maps/dir/?api=1&origin=<lat, long>&destination=<lat, long>"

get_site(url) = take!(Downloads.download(url, IOBuffer()))
get_json(url) = JSON3.read(get_site(url))

function lat_long_pair(n)
    return (n["lat"], n["lon"])
end

function to_df(lat, lon)
    query = gen_query(lat, lon)
    response = HTTP.post(url, nothing, query)
    j = JSON3.read(response.body)
    es = j.elements
    isempty(es) && error("no results")

    ps = lat_long_pair.(es)
    distances = haversine.(ps, ((lat, lon),))
    # sorted_ns = es[sortperm(distances)]
    df = DataFrame(es)
    df.distance = sort(distances)
    urls = map(x -> generate_google_api_url(lat, lon, x...), ps)
    df.maps_url = urls
    df 
end

function node_string(tag)
    if haskey(tag, :value)
        """[\"$(tag.key)\"=\"$(tag.value)\"];"""
    else
        """[\"$(tag.key)\"];"""
    end
end

function generate_google_api_url(origin_lat,origin_long, dest_lat, dest_long) 
    URI(MAPS_URI; query=Dict("api" => 1, "origin" => "$origin_lat,$origin_long", "destination" => "$dest_lat,$dest_long", "dir_action" => "navigate", "travelmode" => "walking")) 
end

@get "/{lat}/{lon}" function foo(req::HTTP.Request, lat, lon)
    lat = parse(Float64, lat)
    lon = parse(Float64, lon)
    df = to_df(lat, lon)
    view_df = df[:, [:id, :distance, :lat, :lon, :maps_url]]
    sort!(view_df, :distance)
    io = IOBuffer()
    pretty_table(io, view_df, nosubheader=true, backend=Val(:html))
    
    HTTP.Response(200, ["Content-Type" => "text/html"]; body=take!(io))
end

@get "/" function (req::HTTP.Request)
    HTTP.Response(200, ["Content-Type" => "text/html"]; body="""
    <html>
    <head>
    <title>Free2Pee</title>
    </head>
    <body>
    <h1>Free2Pee</h1>
    <p>Find the nearest public toilets</p>
    <form action="/$LAT/$LON">
    <input type="submit" value="Submit">
    </form>
    </body>
    </html>
    """)
end

# serve(;host="10.0.0.224")
serve()

# try this not haversine
# curl 'http://router.project-osrm.org/route/v1/driving/13.388860,52.517037;13.397634,52.529407;13.428555,52.523219?overview=false'
