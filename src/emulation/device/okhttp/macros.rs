macro_rules! mod_generator {
    ($mod_name:ident, $cipher:expr, $ua:expr) => {
        pub(crate) mod $mod_name {
            use super::*;
            #[inline(always)]
            pub fn emulation(option: EmulationOption) -> Emulation {
                build_emulation(option, $cipher, $ua)
            }
        }
    };
}
