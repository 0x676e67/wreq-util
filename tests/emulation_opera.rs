#[macro_use]
mod support;

use support::CLIENT;
use wreq_util::Emulation;

// Enabling certain extensions will change the length during encryption. This is because TLS will automatically use padding to fill the data and add a padding extension. At this time, the ja4 fingerprint will change.

test_emulation!(
    test_opera116,
    Emulation::Opera116,
    ["t13d1516h2_8daaf6152771_02713d6af862"],
    "52d84b11737d980aef856699f885ca86"
);

test_emulation!(
    test_opera117,
    Emulation::Opera117,
    ["t13d1516h2_8daaf6152771_02713d6af862"],
    "52d84b11737d980aef856699f885ca86"
);

test_emulation!(
    test_opera118,
    Emulation::Opera118,
    ["t13d1516h2_8daaf6152771_02713d6af862"],
    "52d84b11737d980aef856699f885ca86"
);

test_emulation!(
    test_opera119,
    Emulation::Opera119,
    ["t13d1516h2_8daaf6152771_02713d6af862"],
    "52d84b11737d980aef856699f885ca86"
);
