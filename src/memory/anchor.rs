// Copyright (c) Cole Frederick. All rights reserved.
// The use and distribution terms for this software are covered by the
// Eclipse Public License 1.0 (https://opensource.org/licenses/eclipse-1.0.php)
// which can be found in the file epl-v10.html at the root of this distribution.
// By using this software in any fashion, you are agreeing to be bound by the terms of this license.
// You must not remove this notice, or any other, from this software.

use memory;
use memory::unit::Unit;

pub struct Anchor {
    pub unit: Unit,
}

impl Anchor {
    pub fn for_capacity(capacity: u32) -> Anchor {
        let ensure_odd = (capacity << 1) | 1u32;
        // ensure_odd is distinct from a pointer,
        // by virtue of being odd
        Unit::from(ensure_odd).into()
    }

    pub fn capacity(&self) -> u32 {
        let c: u32 = self.unit.into();
        c >> 1
    }

    pub fn alias_field(&self) -> u32 {
        let a: u64 = self.unit.into();
        (a >> 32) as u32
    }

    pub fn with_alias_field(&self, f: u32) -> Anchor {
        let c: u32 = self.unit.into();
        Anchor { unit: Unit::from(((f as u64) << 32) | (c as u64)) }
    }
}

impl From<Unit> for Anchor {
    fn from(u: Unit) -> Self {
        Anchor { unit: u }
    }
}

pub struct AnchorLine {
    pub line: *const Anchor,
}

impl AnchorLine {
    pub fn set_anchor(&self, a: Anchor) {
        memory::set(Unit::from(self.line).into(), a.into())
    }

    pub fn get_anchor(&self) -> Anchor {
        let anchor_or_ptr =
            memory::get(Unit::from(self.line).into());
        if anchor_or_ptr.is_even() {
            memory::get(anchor_or_ptr.into()).into()
        } else {
            anchor_or_ptr.into()
        }
    }
}

impl From<Unit> for AnchorLine {
    fn from(u: Unit) -> Self {
        AnchorLine { line: u.into() }
    }
}
