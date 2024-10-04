## NOTE:

A re-imagining of the backend is underway. I came to the realization that this design I came up with
for a completely tweakable closed-source codebase is no longer necessary now that the source is open,
and was holding me back from getting something working, as well as preventing the use of most of what
makes Rust so great. As a result, I've scrapped it all. I'm starting (mostly) from scratch, and pulling
in functions as I need them, but the architecture is WAY simpler now, and I expect to have a working
product much sooner.

This new architecture is going to employ a ticking-style event loop that processes commands with a
deadline and then pushes a frame. Layers are also getting some new limitations in that their signatures
follow a similar design to a graphics shader, which will allow them to be heavily parallelized, and
potentially able to make use of GPU hardware down the line. Layers also already have recursion abilities,
which can be used to make things like presets, which are smaller subsets of values that you may want to
combine when building a submaster.

Check it out on branch `redesign-2`!

# SimplyDMX

SimplyDMX aims to be a performant, reliable, and simple program for stage lighting that is highly modular
and easy to use. I'm not a big fan of the current state of the stage lighting software industry, and I
want to change it. Some programs are easy to use, some are powerful, some are good at effects, some at
timecoding, and some at live control. Across these strengths though, there isn't much overlap. My goal
with SimplyDMX is to make something modular that can fit all of these, with the primary focus being on
live control.

SimplyDMX's backend logic is written in Rust and controlled via an RPC API. It is very performant (though
I have plans to make it better) and can run on fairly low-end hardware. The main prerequisite at the
moment is that the target machine must be able to run a web app. The frontend is written in Vue.JS and
TypeScript, with RPC API bindings automatically generated from Rust.

The UI is designed to be very unique, but simple and intuitive. Every UI element is hand-crafted from
scratch with a modern take on a retro neon style that leverages glowing effects to show focus.

SimplyDMX is designed to be my software of choice going forward. It's taken a ***LONG*** time, but I
believe I've built a good enough foundation that I will be able to make exponential progress, and things
will get easier as I continue to build out the lower levels of abstraction. This means that myself and
future authors will have an easy, high-level API to work with that allows easy addition of almost any
feature with seamless integration with the rest of the program.

More details to come.


## Testing

The default frontend for SimplyDMX is a Tauri app. Tauri is like Electron, but lighter, with a better
build system, and a native Rust backend instead of a NodeJS engine. This allows me to write a consistent,
cross-platform UI with significantly less effort and fewer compromises. I would like to create truly
native UIs later on, but for now, Tauri is the best option for widest compatability with minimal effort.

To get started, make sure you have NodeJS and a Rust toolchain installed.

Then, install the Tauri CLI using:

```bash
cargo install tauri-cli
```

Now you can cd into the `simplydmx_frontend` directory, install JavaScript dependencies, and run
the Tauri app. This will build the frontend, start a dev server, build the SimplyDMX library/backend,
the Tauri frontend binary, then start SimplyDMX's web UI.

```bash
cd simplydmx_frontend
npm install # or yarn, whichever you prefer
cargo tauri dev
```

## Current Progress

For progress updates, check out the wiki:
https://github.com/sploders101/simplydmx/wiki
