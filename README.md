
# litedasm

Litedasm is a configurable and extendable assembler.
It allows the user to customize the parser using a configuration file (`arch.ron`).
Additionally it is possible to define symbols, exceptions and other program specific 
details in an additional configuration file (`ctx.ron`).
The configuration files use the `ron` file format.

Currently the following built in architectures are availble:

- 6502 (no unofficial opcodes)
- 65c02
- 65c816 (can switch between 16-bit and 8-bit mode by setting the `m` or `x` flags)

## Table of content

- [Installation](#Installation)
- [Usage](#Usage)
- [License](#License)
- [Contributing](#Contributing)
- [TODO](#TODO)

## Installation

This program requires the latest stable release of rust.
Once rust is set up correclty simply clone the repository.
Then run:

```sh
cargo build # to build or
cargo install  # to install 
```

## Usage

Litedasm currently has a simple command line interface.
Help is available by running:
```sh 
litedasm --help 
```
The `arch` and `ctx` files can be dumped by using the following commands:
```sh
litedasm dump-ctx
litedasm dump-arch 
```
Those outputs can be used as starting points for a custom configuration.


## License

This program is distributed under the terms of the MIT or Apache License.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion 
in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, 
without any additional terms or conditions.

## TODO

