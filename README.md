# Mercator deformation maps

This is a rust conversion of the deformation maps support, as a library, for integration within Mercator.

The reference implementation is available on [GitHub](https://github.com/brainvisa/aims-free/) as part of **AIMS**.

## Mercator: Spatial Index

**Mercator** is a spatial *volumetric* index for the [Human Brain Project](http://www.humanbrainproject.eu). It is a component of the [Knowledge Graph](http://www.humanbrainproject.eu/en/explore-the-brain/search/) service, which  provides the spatial anchoring for the metadata registered as well as processes the volumetric queries.

It is build on top of the Iron Sea database toolkit.

## Requirements

### Software

 * Rust: https://www.rust-lang.org

## Run the tests

You can run the tests using:

```sh
cargo test
```

**Note**

 * These tests compare this implementation with the reference implementation. 
 * They assume the reference implementation provides correct results for its intended uses. 
 * A test is passing if the two implementations generate the same outputs.
 * The reference deformation map & test data is **NOT** included, but assumed to be available under `/data`.

## Documentation

For more information, please refer to the [documentation](https://epfl-dias.github.io/mercator_deformation_maps/).

If you want to build the documentation and access it locally, you can use:

```sh
cargo doc --open
```

## Acknowledgements

This open source software code was developed in part or in whole in the
Human Brain Project, funded from the European Union’s Horizon 2020
Framework Programme for Research and Innovation under the Specific Grant
Agreement No. 785907 (Human Brain Project SGA2).
