# Live Controller Design Notes

Just started another rewrite... again... -_-

Leaving this here to serve as a record in case I forget what I was doing.


## Old design

The old design was somewhat stream-based, but with LOADS of dynamic dispatch. This seemed like
a brilliant idea at the time, but quickly got out of hand. I was trying to create a fully-composable
system of transform actions that could be wrapped around one another based on a configuration file.
Essentially, a MIDI message would come in and trigger a function in the `MidiNote` or `MidiCC`
control structures. Let's say it was a touch event on a fader. On the Behringer X-Touch Compact,
these messages come in as CC messages, but in reality they should actually be treated as binary
notes. This message would trigger a function in the `MidiCC` struct, which would see that it has an
action defined on it, and invoke it. This action would be a dynamic dispatch vtable for 
`AnalogToBoolean`, which would itself see that it has an action bound, and invoke *that*. This
action, for example, could be a `LayerSet` action which would then see this value coming from the
touch interface and add the associated layer into the blending queue. Conceptually, this sounded
great! Small, composable pieces that could be set and forgotten. In reality, it was a complicated
nightmare. Lots of dynamic dispatch, wrappers around wrappers around wrappers, and unclear order of
operations concerning events, feedback, linking, and unlinking put the cracks on display. This
design was meant to *avoid* complexity by shifting the burden of common tasks like debouncing to
the controls framework, leaving the interfaces to worry about bindings only.


## New design

The old system set out to *remove* complexity, but only created more of its own. I'm giving up on
that pipe dream and creating single-layer composition. A single function will be responsible for
converting something like a MIDI event into a control event and dispatching to the proper interface.

In order to eliminate complexity, tools will be provided for assisting with things like debouncing
and type conversion, but it is up to the function to implement them. This will allow for quick and
reliable linking/unlinking, as well as better controller-specific heuristics.

In the new design, a board's control is defined only by a set of capabilities and two streams:
input and feedback. Things like debouncing are handled by the control's implementation.
