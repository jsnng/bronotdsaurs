#[macro_export]
macro_rules! span {
    ($( $(#[$attr:meta])* $name:ident ),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy)]
            $(#[$attr])*
            pub struct $name<'a> { pub bytes: &'a [u8] }
        )*
    };
}

#[macro_export]
macro_rules! proof_constructor_never_panics {
    ($harness:ident, $span:ty, $n:literal) => {
        #[cfg(kani)]
        #[kani::proof]
        #[kani::unwind($n)]
        fn $harness() {
            let bytes: [u8; 16] = kani::any();
            let slice = kani::slice::any_slice_of_array(&bytes);
            if let Ok(span) = <$span>::new(slice) {
                for _ in &span {}
            }
        }
    };
}

#[macro_export]
macro_rules! kani_error_stubbed {
    ($e:expr) => {{
        #[cfg(not(kani))]
        { $e }
        #[cfg(kani)]
        { $crate::tds::decoder::error::DecodeError::KaniErrorStub }
    }};
}