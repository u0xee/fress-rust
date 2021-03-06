// Copyright (c) Cole Frederick. All rights reserved.
// The use and distribution terms for this software are covered by the
// Eclipse Public License 1.0 (https://opensource.org/licenses/eclipse-1.0.php)
// which can be found in the file epl-v10.html at the root of this distribution.
// By using this software in any fashion, you are agreeing to be bound by the terms of this license.
// You must not remove this notice, or any other, from this software.

use std::fmt;
use std::cmp::Ordering;
use memory::*;
use dispatch::*;
use handle::Handle;

pub mod guide;
use self::guide::Guide;

pub struct Uuid_ { }
pub fn prism_unit() -> Unit { mechanism::prism::<Uuid_>() }
pub fn is_prism(prism: AnchoredLine) -> bool { prism[0] == prism_unit() }
pub fn find_prism(h: Handle) -> Option<AnchoredLine> { h.find_prism(prism_unit()) }
pub fn is_uuid(h: Handle) -> bool { find_prism(h).is_some() }

pub fn get_bit(x: u64, y: u64, idx: u8) -> u32 {
    let z = x ^ y;
    let word_idx = (idx & 0x3F) as u64;
    let x_ = (x >> word_idx) as u32;
    let z_ = (z >> word_idx) as u32;
    let masked = z_ & (idx as u32 >> 6);
    (masked ^ x_) & 0x01
}
pub const HEX: (u64, u64) = (0x_03FF_0000_0000_0000, 0x_0000_007E_0000_007E); // 0123456789 afAF
pub fn hit(b: u8, pattern: (u64, u64)) -> bool {
    get_bit(pattern.0, pattern.1, b) == 1
}
pub fn is_hex(b: u8) -> bool { hit(b, HEX) }
pub fn as_hex(b: u8) -> u8 {
    let d: u8 = if b <= b'9' { b - b'0' }
        else if b <= b'F' { b - b'A' + 10 } else
        { b - b'a' + 10 };
    d
}
pub fn ascii(b: u8) -> bool { (b & 0x80) == 0x00 }

pub fn gather(hex_digits: &[u8]) -> Option<u64> {
    let mut x = 0u64;
    for i in hex_digits.iter() {
        let c = *i;
        assert!(ascii(c));
        if !is_hex(c) { return None }
        let d = as_hex(c);
        assert!(d < 16);
        x = (x << 4) + d as u64;
    }
    Some(x)
}

// F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6
// 32       16   16   16   48      bits
// 36 bytes, 4 dashes, 32 hex digits

pub fn parse(source: &[u8]) -> Result<(u64, u64), String> {
    let should_be = "A uuid should be 32 hex digits with 4 dashes dividing them into groups of \
                          (8 4 4 4 12), like: #uuid \"F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6\".";
    if source.len() != 36 {
        return Err(format!("Bad uuid, too {}. {}", if source.len() < 36 { "short" } else { "long" }, should_be))
    }
    if source[8] != b'-' || source[13] != b'-' || source[18] != b'-' || source[23] != b'-' {
        return Err(format!("Bad uuid, groups aren't right. {}", should_be))
    }
    let top = {
        let time_low = match gather(&source[0..8]) {
            Some(x) => { x },
            None => { return Err(format!("Bad uuid, first group contains non-hex characters. {}", should_be)) },
        };
        let time_mid = match gather(&source[9..13]) {
            Some(x) => { x },
            None => { return Err(format!("Bad uuid, second group contains non-hex characters. {}", should_be)) },
        };
        let time_high = match gather(&source[14..18]) {
            Some(x) => { x },
            None => { return Err(format!("Bad uuid, third group contains non-hex characters. {}", should_be)) },
        };
        (time_low << 32) | (time_mid << 16) | time_high
    };
    let bot = {
        let clock_seq = match gather(&source[19..23]) {
            Some(x) => { x },
            None => { return Err(format!("Bad uuid, fourth group contains non-hex characters. {}", should_be)) },
        };
        let node = match gather(&source[24..36]) {
            Some(x) => { x },
            None => { return Err(format!("Bad uuid, fifth group contains non-hex characters. {}", should_be)) },
        };
        (clock_seq << 48) | node
    };
    Ok((top, bot))
}

pub fn new_parsed(source: &[u8]) -> Result<Handle, String> {
    let (top, bot) = match parse(source) {
        Err(msg) => { return Err(msg) },
        Ok(x) => x,
    };
    let needed = 1 /*prism*/ + Guide::units();
    let s = Segment::new(needed);
    let prism = s.line_at(0);
    prism.set(0, prism_unit());
    let guide = Guide { hash: 0, top, bot, prism };
    Ok(guide.store().segment().unit().handle())
}

impl Dispatch for Uuid_ { /*default tear_down, alias_components*/ }
impl Identification for Uuid_ {
    fn type_name(&self) -> &'static str { "Uuid" }
}
impl Distinguish for Uuid_ {
    fn hash(&self, prism: AnchoredLine) -> u32 {
        let guide = Guide::hydrate(prism);
        if guide.has_hash() { return guide.hash; }

        use hash::hash_128;
        let h = hash_128(guide.top, guide.bot, 16);
        guide.set_hash(h).store_hash().hash
    }
    fn eq(&self, prism: AnchoredLine, other: Unit) -> bool {
        let o = other.handle();
        if let Some(o_uuid) = find_prism(o) {
            let g = Guide::hydrate(prism);
            let h = Guide::hydrate(o_uuid);
            g.top == h.top && g.bot == h.bot
        } else {
            false
        }
    }
    fn cmp(&self, prism: AnchoredLine, other: Unit) -> Option<Ordering> {
        // sort by time fields
        unimplemented!()
    }
}
impl Aggregate for Uuid_ { }
impl Sequential for Uuid_ { }
impl Associative for Uuid_ { }
impl Reversible for Uuid_ {}
impl Sorted for Uuid_ {}

pub fn field(width: u32) -> u64 { (1 << width as u64) - 1 }

impl Notation for Uuid_ {
    fn edn(&self, prism: AnchoredLine, f: &mut fmt::Formatter) -> fmt::Result {
        let guide = Guide::hydrate(prism);
        let time_low = (guide.top >> 32) & field(32);
        let time_mid = (guide.top >> 16) & field(16);
        let time_high = guide.top & field(16);
        let clock_seq = (guide.bot >> 48) & field(16);
        let node = guide.bot & field(48);
        write!(f, "#uuid \"{:08X}-{:04X}-{:04X}-{:04X}-{:012X}\"",
               time_low, time_mid, time_high, clock_seq, node)
    }
}

impl Numeral for Uuid_ {}
impl Callable for Uuid_ {}

#[cfg(test)]
mod tests {
    use super::*;

}
