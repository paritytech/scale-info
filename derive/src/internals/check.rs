use super::{ast::Container, Ctxt};

/// Cross-cutting checks that require looking at more than a single attrs
/// object. Simpler checks should happen when parsing and building the attrs.
pub fn check(cx: &Ctxt, cont: &mut Container) {
    // TODO: do we need any checks of this kind? I think not, but maybe parity-scale-codec
}
