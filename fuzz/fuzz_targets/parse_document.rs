#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    // Include malformed UTF-8, decoded exactly as text arriving from a file.
    let source = String::from_utf8_lossy(data);
    let _ = frees_core::parse_document(&source);
});
