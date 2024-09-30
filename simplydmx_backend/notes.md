# Architecture & Design Notes

Every lighting software defines common terms in different ways, so to eliminate confusion, I want to define the terms used in SimplyDMX here.

SimplyDMX is what I call a "layering lighting program." At the core of its calculations is the blending engine, which may also be referred to as the **mixer**. Everything in SimplyDMX is a layer, and layers function similar to graphics shaders. They get a context for the current frame, the previous value, the current layer, and the channel ID to calculate.

This is currently iteration 5 of SimplyDMX, and essentially a completely redesigned system. I have ditched the plugin system, dynamic dispatch, and polymorphism. I have ditched the many tasks that all have different jobs like mixer, patcher, DMX serialization, and individual drivers, and instead am going to a single-threaded tick system that delegates to thread-pools via rayon for compute-heavy tasks that can be parallelized. This drastically simplifies the control flow, and makes it significantly simpler to tighten up timing of frames.


## Definitions


### Submaster

A submaster is a collection of values that can be used in layering. It contains a `shade` function capable of applying the submaster's values on top of a base layer for a single channel. The shade function is intended to be highly parallelized.


### Layer

A layer is a single item in a layer stack which references a submaster and an opacity value, and is used for blending. This indirection allows for the ability to create multiple independent controls using the same submaster, similar to Lightkey. The submaster contains the values, and the layer determines the order & opacity with which it is applied.


### Blender

The blender is the overarching layer manager which juggles layer stacks to enable things like "blind mode."
