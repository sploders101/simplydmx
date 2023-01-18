# Mixer

The mixer blends all of the show fragments together. It holds the values for all submasters,
FX, and anything else that feeds into the lights, and makes sure they all get blended together
into coherent looks that can be output to the lights.


## Architecture

The architecture of the mixer is made of three main parts: submasters, layers, and mixing contexts.
A mixing context can be used recursively within a layer, but the mixer only handles the master context.


## Submasters

The mixer resolves conflicts by blending together fragments called "submasters". A submaster is the
raw data that should be included during blending, and is a `HashMap<Uuid, HashMap<String, u16>>`.
The outer hash map maps fixture IDs to another hash map, which contains associations between
channels (red, green, blue, intensity, etc) and u16 values. If a value does not exist within the
submaster, it will not affect the output for that channel in any way when the submaster is blended.


## Layers

A layer is an association between submaster, layer bin, and opacity. The values of the submaster are
used, laid on top of the current look at the specified opacity, and feed into the result of the layer
bin.


## Layer Bins

Layer bins provide a way for the application to transition between two states. They are opaque by
default, meaning they are purely for transition, rather than mixing. In other words, If a submaster
is not set in the top layer bin, it will fade to 0.


### Justification for opaque requirement

Adding too many variables to the application drastically increases development time. While I'd love
to provide the extra flexibility, this is something that can be done with limitations upfront, and
once an MVP made with the new architecture is finished, a new mixer can be built with an extended
API. Thanks to the loosely-standardized plugin framework that uses dynamic types and lookup tables,
the original API can be kept, while the internals get changed to support more features, without
breaking any dependent plugins.
