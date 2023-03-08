#[macro_export]
macro_rules! inscribe_runes {
    ($path:literal) => {
        static __PRIVATE_INNER_RUNES_DATA: &'static [u8] = include_bytes!($path);
        pub unsafe fn get_runes() -> &'static runes::ArchivedRunes {
            rkyv::archived_root::<runes::Runes>(__PRIVATE_INNER_RUNES_DATA)
        }
    };
}
