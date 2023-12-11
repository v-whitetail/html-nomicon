# html-nomicon #

A Rust-Based Utility to Populate HTML Templates with JSON

## Goals ##

This binary serves as a utility to ship with the [Fusion-360-Report-Viewer](https://github.com/v-whitetail/Fusion-360-Report-Viewer).

The scope of this project is currently limited to accpeting a buffer as JSON from an external application and populating HTML documents.

The buffer may be supplied as an optional argument to this binary or via StdIn.

This binary anticipates a Templates directory, a Reports directory, and a Resources directory.

It is secondary to [tcp_localhost](https://github.com/v-whitetail/tcp_locahost).

As such, it will not build missing components on startup.

## Function ##

This binary is intended to be called as an embedded CLI. 

It expects 3 arguments:

1. A path to the working directory. This Should be the same as the path supplied to tcp_localhost.

2. An optional string of valid JSON.

3. An optional filepath.

Note: If neighter 2 nor 3 are supplied, this binary expects the buffer via StdIn.

The public API for this binary provides the Data::get() and Data::get_with_timeout() methods to process arguments.

Data::get_with_timeout() will close with an error after 16 seconds if no data is passed in.

Since this binary is indented to be called by an application, this is done to prevent multiple instances running continuously.
