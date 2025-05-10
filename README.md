# ASIAIR Rust crate

This library provides a Rust crate to interact with ZWO ASIAir devices. It manage the network protocol to communicate
with them and provides a ideomatic Rust API to interact with instances.

As a side effect it also implements an ASIAir Simulator that enables the automated testing of the library against a software
model.

The top crate is a workspace with two child crates:

- lib: implements the asiair crate
- sim: implements the asisim crate

