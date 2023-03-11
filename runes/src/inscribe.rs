#[macro_export]
macro_rules! inscribe_runes {
    ($path:literal) => {
        #[repr(C)] // guarantee 'bytes' comes after '_align'
        pub struct AlignedAsPrivate<Align, Bytes: ?Sized> {
            pub _align: [Align; 0],
            pub bytes: Bytes,
        }

        #[macro_export]
        macro_rules! include_bytes_align_as {
            ($align_ty:ty, $_path:literal) => {{
                // const block expression to encapsulate the static
                use crate::AlignedAsPrivate;

                // this assignment is made possible by CoerceUnsized
                static ALIGNED: &AlignedAsPrivate<$align_ty, [u8]> = &AlignedAsPrivate {
                    _align: [],
                    bytes: *include_bytes!($path),
                };

                &ALIGNED.bytes
            }};
        }

        static __PRIVATE_INNER_RUNES_DATA: &'static [u8] = include_bytes_align_as!(u64, $path);
        pub unsafe fn get_runes_unchecked() -> &'static runes::ArchivedRunes {
            rkyv::archived_root::<runes::Runes>(__PRIVATE_INNER_RUNES_DATA)
        }
        pub unsafe fn get_runes() -> Option<&'static runes::ArchivedRunes> {
            rkyv::check_archived_root::<runes::Runes>(__PRIVATE_INNER_RUNES_DATA).ok()
        }
    };
}
