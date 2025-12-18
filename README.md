
# wreq-util

A collection of utilities to do common things with [wreq](https://github.com/0x676e67/wreq).

[![Crates.io](https://img.shields.io/crates/v/wreq-util.svg?logo=rust)](https://crates.io/crates/wreq-util)
[![Documentation](https://docs.rs/wreq-util/badge.svg)](https://docs.rs/wreq-util)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL3.0-blue.svg)](./LICENSE)

See the [crate documentation](https://docs.rs/wreq-util/latest/wreq_util) for more details.

## Overview

**wreq-util** offers a set of [tower](https://github.com/tower-rs/tower) middleware and utilities designed specifically for the [wreq](https://github.com/0x676e67/wreq) HTTP client:

- **Emulation** various mainstream browsers (Chrome, Firefox, Safari, Opera, OkHttp) and their versions.

- **Delay/JitterDelay**: Add fixed or jittered delays to HTTP [request](https://docs.rs/http/latest/http/request/index.html)/[response](https://docs.rs/http/latest/http/response/index.html), with customizable strategies and predicates.

## Compatibility

- Integrates with the [tower-service](https://github.com/tower-rs/tower) ecosystem and can be combined with other [tower](https://github.com/tower-rs/tower) middleware.
- Suitable for [wreq](https://github.com/0x676e67/wreq) HTTP client project.

## Contributing

Contributions are welcome! Please open submit pull requests on the [GitHub repository](https://github.com/0x676e67/wreq-util/pulls).

## License

**wreq-util** © [0x676e67](https://github.com/0x676e67), Released under the [GPL-3.0](https://github.com/0x676e67/wreq-util/blob/main/LICENSE) License.
