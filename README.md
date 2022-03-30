# Post Compile Include

A library to generate rust binaries that contain large DATA sections
with unique data that can then be quickly/conveniently modified after compilation.

at a high level, this library includes functions that can be ran during your compilation (read: build.rs) that include a large amount of specific data (eg: a bunch of 'q's) into your compiled file.

It also includes functions that can be ran by an external program and look for that specific data (ie: it searches for a massive wall of 'q's), and it can replace a big chunk of that section with other data that you want included.

It also includes functions that can be run by the program itself, to read from its own data section and search for the modified data, and extract the portions it needs.

See the `examples/` directory for a sample workflow of how this can be used.
