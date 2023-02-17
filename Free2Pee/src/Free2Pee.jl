module Free2Pee
using JSON3, Downloads
get_site(url) = take!(Downloads.download(url, IOBuffer()))
get_json(url) = JSON3.read(get_site(url))

DATADIR = joinpath(@__DIR__, "../data/")
# DATADIR2 = joinpath(@__DIR__, "../data/cambridge_redline_places_json/")
data(x) = joinpath(DATADIR, x)

export get_site, get_json

end # module Free2Pee
