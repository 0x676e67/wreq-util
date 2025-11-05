# wreq-util

[![CI](https://github.com/0x676e67/wreq-util/actions/workflows/ci.yml/badge.svg)](https://github.com/0x676e67/wreq-util/actions/workflows/ci.yml)
[![MIT licensed](https://img.shields.io/badge/license-GPL3.0-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/wreq-util.svg?logo=rust)](https://crates.io/crates/wreq-util)
[![docs.rs](https://img.shields.io/docsrs/wreq-util?logo=rust)](https://docs.rs/wreq-util)
![Crates.io Total Downloads](https://img.shields.io/crates/d/wreq-util)

A collection of utilities to do common things with [wreq](https://github.com/0x676e67/wreq).

## Emulation

- **Emulation Device**

  Most browser device models share the same TLS and HTTP/2 configuration, differing only in the User-Agent.

  | Device | Versions |
  |---------|----------|
  | <span style="display:inline-flex; align-items:center;"><img src="https://raw.githubusercontent.com/alrra/browser-logos/main/src/chrome/chrome_32x32.png" width="16" height="16"> Chrome</span> | `Chrome100`, `Chrome101`, `Chrome104`, `Chrome105`, `Chrome106`, `Chrome107`, `Chrome108`, `Chrome109`, `Chrome110`, `Chrome114`, `Chrome116`, `Chrome117`, `Chrome118`, `Chrome119`, `Chrome120`, `Chrome123`, `Chrome124`, `Chrome126`, `Chrome127`, `Chrome128`, `Chrome129`, `Chrome130`, `Chrome131`, `Chrome132`, `Chrome133`, `Chrome134`, `Chrome135`, `Chrome136`, `Chrome137`, `Chrome138`, `Chrome139`, `Chrome140`, `Chrome141`, `Chrome142` |
  | <span style="display:inline-flex; align-items:center;"><img src="https://raw.githubusercontent.com/alrra/browser-logos/main/src/edge/edge_32x32.png" width="16" height="16"> Edge</span> | `Edge101`, `Edge122`, `Edge127`, `Edge131`, `Edge134` |
  | <span style="display:inline-flex; align-items:center;"><img src="https://raw.githubusercontent.com/alrra/browser-logos/main/src/opera/opera_32x32.png" width="16" height="16"> Opera</span> | `Opera116`, `Opera117`, `Opera118`, `Opera119` |
  | <span style="display:inline-flex; align-items:center;"><img src="https://raw.githubusercontent.com/alrra/browser-logos/main/src/safari/safari_32x32.png" width="16" height="16"> Safari</span> | `SafariIos17_2`, `SafariIos17_4_1`, `SafariIos16_5`, `Safari15_3`, `Safari15_5`, `Safari15_6_1`, `Safari16`, `Safari16_5`, `Safari17_0`, `Safari17_2_1`, `Safari17_4_1`, `Safari17_5`, `Safari18`, `SafariIPad18`, `Safari18_2`, `SafariIos18_1_1`, `Safari18_3`, `Safari18_3_1`, `Safari18_5`, `Safari26` |
  | <span style="display:inline-flex; align-items:center;"><img src="https://raw.githubusercontent.com/alrra/browser-logos/main/src/firefox/firefox_32x32.png" width="16" height="16"> Firefox</span>    | `Firefox109`, `Firefox117`, `Firefox128`, `Firefox133`, `Firefox135`, `FirefoxPrivate135`, `FirefoxAndroid135`, `Firefox136`, `FirefoxPrivate136`, `Firefox139`, `Firefox142`, `Firefox143` |
  | OkHttp | `OkHttp3_9`, `OkHttp3_11`, `OkHttp3_13`, `OkHttp3_14`, `OkHttp4_9`, `OkHttp4_10`, `OkHttp4_12`, `OkHttp5` |
