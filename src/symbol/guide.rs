// Copyright (c) Cole Frederick. All rights reserved.
// The use and distribution terms for this software are covered by the
// Eclipse Public License 1.0 (https://opensource.org/licenses/eclipse-1.0.php)
// which can be found in the file epl-v10.html at the root of this distribution.
// By using this software in any fashion, you are agreeing to be bound by the terms of this license.
// You must not remove this notice, or any other, from this software.

use memory::*;

/// The Guide structure is hydrated from its in-memory representation, 64 bits in length.
/// The top 32 bits contain the hash, the bottom 32 bits contain the length in bytes and
/// the position of the prefix separating solidus. A position of 0 means no prefix.

/// `Top 32 bits  [              Hash  (32) ]`
/// `Bottom bits  [ Solidus (8) | Count (8) ]`
///

#[derive(Copy, Clone, Debug)]
pub struct Guide {
    pub hash: u32,

    pub solidus: u32,
    pub count: u32,

    pub prism: AnchoredLine,
    pub root: AnchoredLine,
}

impl Guide {
    pub fn units() -> u32 { if cfg!(target_pointer_width = "32") { 2 } else { 1 } }
    pub fn segment(&self) -> Segment { self.prism.segment() }
    pub fn set_hash(mut self, hash: u32) -> Guide {
        self.hash = hash;
        self
    }
    pub fn clear_hash(mut self) -> Guide {
        self.hash = 0;
        self
    }
    pub fn has_hash(&self) -> bool { self.hash != 0 }
    pub fn reroot(mut self) -> Guide {
        let root_offset = 1 /*prism*/ + Guide::units();
        self.root = self.prism.offset(root_offset as i32);
        self
    }
    pub fn hydrate(prism: AnchoredLine) -> Guide {
        if cfg!(target_pointer_width = "32") {
            Guide::hydrate_top_bot(prism, prism[1].into(), prism[2].into())
        } else {
            let g: u64 = prism[1].into();
            Guide::hydrate_top_bot(prism, (g >> 32) as u32, g as u32)
        }
    }
    pub fn hydrate_top_bot(prism: AnchoredLine, top: u32, bot: u32) -> Guide {
        let hash = top;
        let solidus = (bot >> 8) & 0xFF;
        let count = bot & 0xFF;

        let root_offset = 1 /*prism*/ + Guide::units();
        let root = prism.offset(root_offset as i32);

        Guide { hash, solidus, count, prism, root }
    }

    pub fn new(prism: AnchoredLine, solidus_position: u32, byte_count: u32) -> Guide {
        let root_offset = 1 /*prism*/ + Guide::units();
        Guide { hash: 0, solidus: solidus_position,
            count: byte_count, prism, root: prism.offset(root_offset as i32) }
    }

    pub fn store_at(&self, mut prism: AnchoredLine) {
        let top: u32 = self.hash;
        let bot: u32 = (self.solidus << 8) | self.count;
        if cfg!(target_pointer_width = "32") {
            prism[1] = top.into();
            prism[2] = bot.into();
        } else {
            let g: u64 = ((top as u64) << 32) | (bot as u64);
            prism[1] = g.into();
        }
    }
    pub fn store(self) -> Guide {
        self.store_at(self.prism);
        self
    }
    pub fn store_hash(self) -> Guide {
        let prism = self.prism;
        let top: u32 = self.hash;
        let bot: u32 = (self.solidus << 8) | self.count;
        if cfg!(target_pointer_width = "32") {
            prism.store_hash(1, top.into());
            prism.store_hash(2, bot.into());
        } else {
            let g: u64 = ((top as u64) << 32) | (bot as u64);
            prism.store_hash(1, g.into());
        }
        self
    }

    pub fn byte_slice(&self) -> &mut [u8] {
        use std::slice::from_raw_parts_mut;
        let p = self.root.line().star() as *mut u8;
        unsafe {
            from_raw_parts_mut(p, self.count as usize)
        }
    }
    pub fn str(&self) -> &mut str {
        use std::str::from_utf8_mut;
        from_utf8_mut(self.byte_slice()).unwrap()
    }

    pub fn set_count(mut self, count: u32) -> Guide {
        self.count = count;
        self.clear_hash()
    }
}

