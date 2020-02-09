// Copyright (c) Cole Frederick. All rights reserved.
// The use and distribution terms for this software are covered by the
// Eclipse Public License 1.0 (https://opensource.org/licenses/eclipse-1.0.php)
// which can be found in the file epl-v10.html at the root of this distribution.
// By using this software in any fashion, you are agreeing to be bound by the terms of this license.
// You must not remove this notice, or any other, from this software.

use memory::unit::Unit;
use handle::Handle;
use value::Value;

// structure used for static init, global environment:
// map of all vars, (:kw?)

pub static VAR_SENTINEL: u8 = 0;

/// Var dispatch.
pub struct Var {
    prism: Unit,
}

impl Var {
    pub fn new() -> Unit {
        unimplemented!()
    }
}
