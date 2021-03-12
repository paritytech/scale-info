// TODO: copyright and license
use std::fmt::{self, Display};
use syn::Path;
#[derive(Copy, Clone)]
pub struct Symbol(&'static str);
pub const BOUND: Symbol = Symbol("bound");
pub const SCALE_INFO: Symbol = Symbol("scale_info");

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}
