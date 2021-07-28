use scale_info::TypeInfo;
use scale::Encode;

#[derive(TypeInfo, Encode)]
#[scale_info(capture_docs = "invalid")]
/// Docs
struct InvalidDocsCapture {
    /// Docs
    a: u8,
}

fn main() {}
