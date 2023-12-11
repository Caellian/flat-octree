# Flat Octree

Octree implementation that stores data in a linear chunk of memory which makes
it ideal for rendering applications.

Layout is configurable through generics, but defaults to _breath first_ layout,
which means that values of each depth are grouped together and depths are stored
sequentially one after the next.

## Contributing

Contributions are welcome.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

## License

This project is licensed under [Zlib](./LICENSE_ZLIB), [MIT](./LICENSE_MIT), or
[Apache-2.0](./LICENSE_APACHE) license, choose whichever suits you most.
