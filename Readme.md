# SimplyDMX

This is the plugin-oriented rewrite of SimplyDMX, which focuses on communication between different parts of
the application and splitting out responsibilities among semi-independent plugins.

## Abstract

This rewrite of SimplyDMX is completely modular, and will include many "connectors" which allow it to run in
a flexible manner, such that any client can run as if it were part of the code itself, using serde-based
serialization and a variety of transports. Because of this, the backend can be compiled to run on many different
systems, containing all platform-dependent logic such as MIDI drivers and graphical interfaces in swappable
plugins.

There will be many layers of communication in order to provide maximum flexibility. MIDI can talk to the mixer,
which talks to the DMX renderer, which talks to the DMX driver. In this way, SimplyDMX isn't limited to a
specific DMX transport or adapter, nor is it limited to DMX at all.

## Services

One unique feature of SimplyDMX is the service subsystem. Services provide a common mechanism for plugins to
expose functionality to other plugins in a discoverable, consistent, user-configurable way. Plugins can register
services that power automation and user input devices. For example, the MIDI controller plugin doesn't need to
be programmed to control a light if the mixer plugin can advertise the API. Using the service subsystem, the
MIDI controller plugin can discover a "Toggle Light" service for example, which indicates it takes a `String`
representing a `light-id`. Once the user decides they want a MIDI button to toggle a light, the MIDI controller
plugin can query a list of options for `light-id`, and present them to the user in a neat drop-down. The MIDI
controller plugin has no idea how to toggle a light, but the mixer plugin can hold its hand and walk it through
the configuration, after which the behavior can be delegated by *call*ing the service.

This is all done primarily through two macros:
* The `Service` derive macro
* The `interpolate_service` attribute macro

Using these two macros, all of the boilerplate code necessary for the service to document its signature in a
way that is consistent across all services is written for you. You can have a service up and running in as little
as 17 lines of code, most of which is just descriptions and names for your service and variables, and you don't
need to write *any* of the downcasting/deserializing logic!