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
