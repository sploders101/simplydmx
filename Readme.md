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


## Platform support

SimplyDMX aims to be cross-platform, and many of the decisions I am making are intended to support this
goal. Unfortunately, I don't have unlimited time, and my first priority is to make something *I* want
to use. As of now, SimplyDMX officially supports only MacOS. Once I get an MVP, my top priority will be
expanding support to Linux and Windows.

The current roadblocks to cross-platform support are listed below.

* Midi
    * SimplyDMX currently uses coremidi for MIDI integration. I tried using midir, but it just doesn't
    do what I need it to do. It seems overly-complicated and doesn't support multi-threading, async, or
    unique device IDs. Coremidi supports multi-threading and *persistent* device IDs, which can provide
    a significantly better UX with less effort on the platform I would be using for myself, so that's
    where I've decided to start. I would like to add more platforms, but I'll have to add backends
    myself individually, which takes time.
    * Once I have the controls system implemented, I will refocus on other platforms, either forking midir
    to make it work or trying something like portmidi or rtmidi.
    * Please do not submit any pull requests for new midi backends for the time being. I am still working
    out the design and the midi router's architecture is in flux, so merging in other backends will be
    more work than I'd like to invest at the moment. I will remove this bullet point once the design is
    finished, at which point I will gratefully accept pull requests.


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
