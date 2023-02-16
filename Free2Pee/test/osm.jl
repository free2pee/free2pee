using OpenStreetMapX, Free2Pee, EzXML, XMLDict, CSV, DataFrames
_fn = "osmmap_central.osm"
fn = Free2Pee.data(_fn)
# x = get_map_data(fn)
# y = OpenStreetMapX.parseOSM(fn)

z = readxml(fn)
z.node
n = z.node
d = XMLDict.xml_dict(z)
ns = d["osm"]["node"]

lol = Dict(map(x -> x[:id], ns) .=> ns)
target = "4817053508"
tar = lol[target]["tag"]

tns = ns[haskey.(ns, "tag")]
collect(skipmissing(get_tag.(tns)))

ts = map(x->x["tag"], tns)

function tags_to_dict(tags)
    if tags isa AbstractVector
        return Dict(tag[:k] => tag[:v] for tag in tags)
    else
        # xmldict doesn't always return vec so we special case the single tag case
        return Dict(tags[:k] => tags[:v])
    end
end
toilets_idxs = findall(x->haskey(x, "toilets"), tags_to_dict.(ts))
tls = tns[toilets_idxs]
