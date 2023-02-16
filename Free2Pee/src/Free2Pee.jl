module Free2Pee
using JSON3, Downloads
get_site(url) = take!(Downloads.download(url, IOBuffer()))
get_json(url) = JSON3.read(get_site(url))
const DATADIR = joinpath(@__DIR__, "../data/kendall_places_json/")
data(x) = joinpath(DATADIR, x)
export get_site, get_json

end # module Free2Pee
