const RAPID_SEED: u64 = 0xbdd89aa982704029;
const RAPID_SECRET: [u64; 3] = [0x2d358dccaa6c78a5, 0x8bb84b93962eacc9, 0x4b33a62ed433d4a3];

fn rapid_mul128(a: u64, b: u64) -> (u64, u64) {
    let r = (a as u128) * (b as u128);
    (r as u64, (r >> 64) as u64)
}

fn rapid_mix(a: u64, b: u64) -> u64 {
    let (lo, hi) = rapid_mul128(a, b);
    lo ^ hi
}

fn case_fold(ch: u8) -> u64 {
    if ch.is_ascii_uppercase() {
        (ch - b'A' + b'a') as u64
    } else {
        ch as u64
    }
}

fn cf_read64(p: &[u8]) -> u64 {
    case_fold(p[0])
        | (case_fold(p[1]) << 16)
        | (case_fold(p[2]) << 32)
        | (case_fold(p[3]) << 48)
}

fn cf_read32(p: &[u8]) -> u64 {
    case_fold(p[0]) | (case_fold(p[1]) << 16)
}

fn rapidhash_case_folding(data: &[u8]) -> u64 {
    let p = data;
    let len = data.len() * 2;

    let mut seed = RAPID_SEED;
    seed ^= rapid_mix(seed ^ RAPID_SECRET[0], RAPID_SECRET[1]) ^ (len as u64);

    let (a, b);

    if len <= 16 {
        if len >= 4 {
            let plast = p.len() - 2;
            a = (cf_read32(p) << 32) | cf_read32(&p[plast..]);
            let delta = ((len & 24) >> (len >> 3)) / 2;
            b = (cf_read32(&p[delta..]) << 32) | cf_read32(&p[plast - delta..]);
        } else if len > 0 {
            a = case_fold(p[0]);
            b = 0;
        } else {
            a = 0;
            b = 0;
        }
    } else {
        let mut i = len;
        let mut pp = 0usize;
        seed = if i > 48 {
            let mut see1 = seed;
            let mut see2 = seed;
            let mut s = seed;
            while i >= 48 {
                s = rapid_mix(cf_read64(&p[pp..]) ^ RAPID_SECRET[0], cf_read64(&p[pp + 4..]) ^ s);
                see1 = rapid_mix(
                    cf_read64(&p[pp + 8..]) ^ RAPID_SECRET[1],
                    cf_read64(&p[pp + 12..]) ^ see1,
                );
                see2 = rapid_mix(
                    cf_read64(&p[pp + 16..]) ^ RAPID_SECRET[2],
                    cf_read64(&p[pp + 20..]) ^ see2,
                );
                pp += 24;
                i -= 48;
            }
            s ^ see1 ^ see2
        } else {
            seed
        };

        if i > 16 {
            seed = rapid_mix(
                cf_read64(&p[pp..]) ^ RAPID_SECRET[2],
                cf_read64(&p[pp + 4..]) ^ seed ^ RAPID_SECRET[1],
            );
            if i > 32 {
                seed = rapid_mix(
                    cf_read64(&p[pp + 8..]) ^ RAPID_SECRET[2],
                    cf_read64(&p[pp + 12..]) ^ seed,
                );
            }
        }

        let si = i as isize;
        let a_off = pp as isize + (si - 16) / 2;
        let b_off = pp as isize + (si - 8) / 2;
        a = cf_read64(&p[a_off as usize..]);
        b = cf_read64(&p[b_off as usize..]);
    }

    let a = a ^ RAPID_SECRET[1];
    let b = b ^ seed;
    let (a, b) = rapid_mul128(a, b);
    rapid_mix(a ^ RAPID_SECRET[0] ^ (len as u64), b ^ RAPID_SECRET[1])
}

fn wtf_hash(name: &str) -> u32 {
    let mut r = rapidhash_case_folding(name.as_bytes()) as u32;
    r &= (1u32 << 24) - 1;
    if r == 0 {
        r = 1u32 << 23;
    }
    r
}

const EMPTY: i32 = -1;
const MIN_CAP: usize = 8;

pub(crate) fn header_order<'a>(names: &[&'a str]) -> Vec<&'a str> {
    let n = names.len();
    if n == 0 {
        return Vec::new();
    }

    let hashes: Vec<u32> = names.iter().map(|n| wtf_hash(n)).collect();

    let mut cap = MIN_CAP;
    let mut slots = vec![EMPTY; cap];
    let mut count = 0usize;

    for (idx, &h) in hashes.iter().enumerate() {
        let mask = cap - 1;
        let mut pos = (h as usize) & mask;
        let mut probe = 0usize;
        while slots[pos] != EMPTY {
            probe += 1;
            pos = (pos + probe) & mask;
        }
        slots[pos] = idx as i32;
        count += 1;

        if count * 2 >= cap {
            let old_cap = cap;
            cap *= 2;
            let mut new_slots = vec![EMPTY; cap];
            let mask = cap - 1;
            for i in 0..old_cap {
                let item = slots[i];
                if item == EMPTY {
                    continue;
                }
                let mut pos = (hashes[item as usize] as usize) & mask;
                let mut probe = 0usize;
                while new_slots[pos] != EMPTY {
                    probe += 1;
                    pos = (pos + probe) & mask;
                }
                new_slots[pos] = item;
            }
            slots = new_slots;
        }
    }

    let mut result = Vec::with_capacity(n);
    for &item in &slots {
        if item != EMPTY {
            result.push(names[item as usize]);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_fetch() {
        let order = header_order(&[
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "user-agent",
        ]);
        assert_eq!(
            order,
            &["sec-ch-ua-platform", "user-agent", "sec-ch-ua", "sec-ch-ua-mobile"]
        );
    }

    #[test]
    fn test_fetch_with_custom_headers() {
        let order = header_order(&[
            "x-custom-first",
            "x-custom-second",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "user-agent",
        ]);
        assert_eq!(
            order,
            &[
                "x-custom-second",
                "sec-ch-ua-platform",
                "user-agent",
                "sec-ch-ua",
                "x-custom-first",
                "sec-ch-ua-mobile"
            ]
        );
    }

    #[test]
    fn test_fetch_with_many_custom_headers() {
        let order = header_order(&[
            "x-alpha",
            "x-beta",
            "x-delta",
            "x-epsilon",
            "x-gamma",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "user-agent",
        ]);
        assert_eq!(
            order,
            &[
                "x-alpha",
                "sec-ch-ua-platform",
                "sec-ch-ua",
                "x-gamma",
                "sec-ch-ua-mobile",
                "x-beta",
                "x-epsilon",
                "x-delta",
                "user-agent"
            ]
        );
    }

    #[test]
    fn test_fetch_post_content_type() {
        let order = header_order(&[
            "content-type",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "user-agent",
        ]);
        assert_eq!(
            order,
            &[
                "sec-ch-ua-platform",
                "user-agent",
                "sec-ch-ua",
                "content-type",
                "sec-ch-ua-mobile"
            ]
        );
    }

    #[test]
    fn test_hash_known_values() {
        assert_eq!(wtf_hash("sec-ch-ua"), 0x33F009);
        assert_eq!(wtf_hash("sec-ch-ua-mobile"), 0x9B488D);
        assert_eq!(wtf_hash("sec-ch-ua-platform"), 0xFFBEC3);
        assert_eq!(wtf_hash("user-agent"), 0xC21178);
        assert_eq!(wtf_hash("accept"), 0x592219);
    }
}
