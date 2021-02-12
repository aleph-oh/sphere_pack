# sphere_pack
This repository uses the implementation of sphere packing in spherical-cow to parse descriptions of spheres provided as JSON objects, and provides
the resulting packing density, surface area-to-volume ratio, and number of spheres packed. 

## Usage
As building the binaries from source would require cargo, it's easiest to use `cargo install sphere_pack` to build the binaries for your specific
architecture.

The input format for files to be parsed is a JSON list of objects, where each object has a name property expressible as a String,
a radius property expressible as a floating point value, and a proportion property (which is an integer) with a value between 0 and 255 inclusive.

The output format is also a JSON, with properties of volume fraction (expressed as a proportion, not a percentage), surface area to volume ratio, and sphere count.

## TODO
More configurations, unit tests, criterion benchmarks (?)
