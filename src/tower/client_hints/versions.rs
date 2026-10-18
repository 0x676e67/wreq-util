//! Full version numbers of stable Chrome releases.

/// Operating systems Chrome sends User-Agent client hints on.
#[derive(Clone, Copy)]
pub(super) enum Os {
    Windows,
    MacOS,
    Linux,
    Android,
}

/// The last stable build of every Chrome milestone, as its build number and the patch numbers on
/// `[Windows, macOS, Linux, Android]`.
///
/// Taken from the VersionHistory API, which lists every build that reached the stable channel:
/// <https://versionhistory.googleapis.com/v1/chrome/platforms/win64/channels/stable/versions>
/// (and `mac_arm64`, `linux`, `android` in place of `win64`).
///
/// A milestone without a row has no high-entropy hints, so add a row together with each new Chrome
/// profile.
const CHROME: &[(u16, u16, [u16; 4])] = &[
    (100, 4896, [127, 127, 127, 127]),
    (101, 4951, [67, 64, 64, 61]),
    (102, 5005, [115, 115, 115, 125]),
    (103, 5060, [134, 134, 134, 129]),
    (104, 5112, [102, 101, 101, 97]),
    (105, 5195, [128, 125, 125, 136]),
    (106, 5249, [121, 119, 119, 126]),
    (107, 5304, [123, 121, 121, 141]),
    (108, 5359, [126, 124, 124, 128]),
    (109, 5414, [122, 119, 119, 118]),
    (110, 5481, [180, 177, 177, 154]),
    (111, 5563, [149, 146, 146, 116]),
    (112, 5615, [140, 137, 165, 136]),
    (113, 5672, [129, 126, 126, 163]),
    (114, 5735, [248, 248, 198, 227]),
    (115, 5790, [173, 170, 170, 168]),
    (116, 5845, [190, 187, 187, 173]),
    (117, 5938, [152, 149, 149, 156]),
    (118, 5993, [120, 117, 117, 112]),
    (119, 6045, [202, 199, 199, 194]),
    (120, 6099, [227, 234, 224, 231]),
    (121, 6167, [189, 184, 184, 180]),
    (122, 6261, [131, 129, 128, 120]),
    (123, 6312, [124, 124, 122, 121]),
    (124, 6367, [210, 210, 207, 179]),
    (125, 6422, [176, 176, 141, 186]),
    (126, 6478, [185, 185, 182, 188]),
    (127, 6533, [122, 122, 119, 106]),
    (128, 6613, [139, 139, 137, 148]),
    (129, 6668, [103, 103, 100, 102]),
    (130, 6723, [119, 119, 116, 103]),
    (131, 6778, [267, 267, 264, 261]),
    (132, 6834, [197, 162, 159, 165]),
    (133, 6943, [143, 143, 141, 138]),
    (134, 6998, [179, 167, 165, 136]),
    (135, 7049, [117, 117, 114, 113]),
    (136, 7103, [116, 116, 113, 127]),
    (137, 7151, [122, 122, 119, 117]),
    (138, 7204, [185, 185, 183, 180]),
    (139, 7258, [157, 157, 154, 160]),
    (140, 7339, [210, 215, 207, 208]),
    (141, 7390, [125, 124, 122, 123]),
    (142, 7444, [177, 177, 175, 173]),
    (143, 7499, [194, 194, 192, 194]),
    (144, 7559, [135, 135, 132, 133]),
    (145, 7632, [162, 162, 159, 161]),
    (146, 7680, [180, 180, 177, 178]),
    (147, 7727, [139, 139, 137, 138]),
    (148, 7778, [218, 217, 215, 217]),
    (149, 7827, [201, 201, 200, 201]),
    (150, 7871, [189, 189, 186, 189]),
    (151, 7922, [176, 176, 173, 175]),
    (152, 7977, [85, 85, 82, 84]),
    (153, 8010, [55, 55, 52, 53]),
    (154, 8037, [100, 100, 97, 127]),
];

/// Returns the full version of the given Chrome milestone on `os`.
pub(super) fn chrome_full_version(major: u16, os: Os) -> Option<String> {
    let row = CHROME
        .binary_search_by_key(&major, |(major, ..)| *major)
        .ok()?;
    let (major, build, patches) = CHROME[row];
    Some(format!("{major}.0.{build}.{}", patches[os as usize]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_sorted() {
        for pair in CHROME.windows(2) {
            assert!(pair[0].0 < pair[1].0);
        }
    }

    #[test]
    fn looks_up_by_milestone_and_os() {
        assert_eq!(
            chrome_full_version(154, Os::Linux),
            Some("154.0.8037.97".to_owned())
        );
        assert_eq!(
            chrome_full_version(154, Os::Android),
            Some("154.0.8037.127".to_owned())
        );
        assert_eq!(chrome_full_version(99, Os::Windows), None);
    }
}
