use std::{
    cell::Cell,
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    num::Wrapping,
};

use strum::VariantArray;

use super::{Emulation, EmulationOS, EmulationOption};

// from: https://github.com/seanmonstar/reqwest/blob/44ac897f1ab35ba24a195927043d185d5cbb6912/src/util.rs#L27
fn fast_random() -> u64 {
    thread_local! {
        static RNG: Cell<Wrapping<u64>> = Cell::new(Wrapping(seed()));
    }

    #[inline]
    fn seed() -> u64 {
        let seed = RandomState::new();

        let mut out = 0;
        let mut cnt = 0;
        while out == 0 {
            cnt += 1;
            let mut hasher = seed.build_hasher();
            hasher.write_usize(cnt);
            out = hasher.finish();
        }
        out
    }

    RNG.with(|rng| {
        let mut n = rng.get();
        debug_assert_ne!(n.0, 0);
        n ^= n >> 12;
        n ^= n << 25;
        n ^= n >> 27;
        rng.set(n);
        n.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    })
}

impl Emulation {
    /// Returns a random variant of the `Emulation` enum.
    ///
    /// This method uses a fast random number generator to select a random variant
    /// from the `Emulation::VARIANTS` array. The random number generator is based
    /// on the XOR-Shift algorithm, which is efficient and suitable for use in
    /// multi-threaded environments.
    ///
    /// # Examples
    ///
    /// ```
    /// use wreq_util::Emulation;
    ///
    /// let random_emulation = Emulation::random();
    /// println!("{:?}", random_emulation);
    /// ```
    ///
    /// # Panics
    ///
    /// This method will panic if the `Emulation::VARIANTS` array is empty.
    #[inline]
    pub fn random() -> EmulationOption {
        let emulation = Emulation::VARIANTS;
        let emulation_os = EmulationOS::VARIANTS;
        let rand = fast_random() as usize;
        EmulationOption::builder()
            .emulation(emulation[rand % emulation.len()])
            .emulation_os(emulation_os[rand % emulation_os.len()])
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread,
    };

    use super::*;

    #[test]
    fn test_concurrent_get_random_emulation() {
        const THREAD_COUNT: usize = 10;
        const ITERATIONS: usize = 100;

        let results = Arc::new(Mutex::new(Vec::new()));

        let mut handles = vec![];

        for _ in 0..THREAD_COUNT {
            let results = Arc::clone(&results);
            let handle = thread::spawn(move || {
                for _ in 0..ITERATIONS {
                    let emulation = Emulation::random();
                    let mut results = results.lock().unwrap();
                    results.push(emulation);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let results = results.lock().unwrap();
        println!("Total results: {}", results.len());
    }
}
