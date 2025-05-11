# rquest-util

[![CI](https://github.com/0x676e67/rquest-util/actions/workflows/ci.yml/badge.svg)](https://github.com/0x676e67/rquest-util/actions/workflows/ci.yml)
[![MIT licensed](https://img.shields.io/badge/license-GPL3.0-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/rquest-util.svg?logo=rust)](https://crates.io/crates/rquest-util)
[![docs.rs](https://img.shields.io/docsrs/rquest-util?logo=rust)](https://docs.rs/rquest-util)
![Crates.io Total Downloads](https://img.shields.io/crates/d/rquest-util)

A collection of utilities to do common things with [rquest](https://github.com/0x676e67/rquest).

## Emulation

- **HTTP/2 over TLS**

  **JA3**/**JA4**/**Akamai** fingerprints cannot accurately simulate browser fingerprints due to the sophistication of TLS encryption and the popularity of HTTP/2. rquest-util does not plan to support parsing these fingerprint strings for simulation. Users are encouraged to customize the configuration according to their own needs.

- **Emulation Device**

  Most browser device models share the same TLS and HTTP/2 configuration, differing only in the User-Agent.

    <details>

    <summary>Default device emulation types</summary>

    | **Browser**   | **Versions**                                                                                     |
    |---------------|--------------------------------------------------------------------------------------------------|
    | **Chrome**    | `Chrome100`, `Chrome101`, `Chrome104`, `Chrome105`, `Chrome106`, `Chrome107`, `Chrome108`, `Chrome109`, `Chrome110`, `Chrome114`, `Chrome116`, `Chrome117`, `Chrome118`, `Chrome119`, `Chrome120`, `Chrome123`, `Chrome124`, `Chrome126`, `Chrome127`, `Chrome128`, `Chrome129`, `Chrome130`, `Chrome131`, `Chrome132`, `Chrome133`, `Chrome134`, `Chrome135`, `Chrome136` |
    | **Edge**      | `Edge101`, `Edge122`, `Edge127`, `Edge131`, `Edge134`                                                       |
    | **Safari**    | `SafariIos17_2`, `SafariIos17_4_1`, `SafariIos16_5`, `Safari15_3`, `Safari15_5`, `Safari15_6_1`, `Safari16`, `Safari16_5`, `Safari17_0`, `Safari17_2_1`, `Safari17_4_1`, `Safari17_5`, `Safari18`, `SafariIPad18`, `Safari18_2`, `Safari18_1_1`, `Safari18_3`, `Safari18_3_1` |
    | **OkHttp**    | `OkHttp3_9`, `OkHttp3_11`, `OkHttp3_13`, `OkHttp3_14`, `OkHttp4_9`, `OkHttp4_10`, `OkHttp4_12`, `OkHttp5`         |
    | **Firefox**   | `Firefox109`, `Firefox117`, `Firefox128`, `Firefox133`, `Firefox135`, `FirefoxPrivate135`, `FirefoxAndroid135`, `Firefox136`, `FirefoxPrivate136`|

    </details>

## Contributing

If you would like to submit your contribution, please open a [Pull Request](https://github.com/0x676e67/rquest-util/pulls).

## License

**rquest-util** © [0x676e67](https://github.com/0x676e67), Released under the [GPL-3.0](https://github.com/0x676e67/rquest-util/blob/main/LICENSE) License.
