// Copyright (c) Cole Frederick. All rights reserved.
// The use and distribution terms for this software are covered by the
// Eclipse Public License 1.0 (https://opensource.org/licenses/eclipse-1.0.php)
// which can be found in the file epl-v10.html at the root of this distribution.
// By using this software in any fashion, you are agreeing to be bound by the terms of this license.
// You must not remove this notice, or any other, from this software.

use memory::*;
use dispatch::*;
use super::guide::Guide;
use super::*;

pub fn conj(prism: Line, x: Unit) -> Unit {
    let guide: Guide = prism[1].into();
    let count = guide.count();
    if count <= TAIL_CAP {
        conj_untailed(prism, x, guide, count)
    } else {
        conj_tailed(prism, x, guide, count)
    }
}

fn conj_untailed(prism: Line, x: Unit, guide: Guide, count: u32) -> Unit {
    if count == TAIL_CAP {
        conj_untailed_complete(prism, x, guide, count)
    } else {
        conj_untailed_incomplete(prism, x, guide, count)
    }
}

fn conj_untailed_complete(prism: Line, x: Unit, guide: Guide, count: u32) -> Unit {
    let mut tail = Segment::new(TAIL_CAP);
    tail[1] = x;
    let anchor_gap = guide.prism_to_anchor_gap();
    let root_gap = guide.guide_to_root_gap();

    let mut segment: Segment = prism.offset(-((anchor_gap + 1) as isize)).into();
    let mut s = if segment.is_aliased() {
        unalias_root(segment, anchor_gap, root_gap, count, guide)
    } else { segment };
    let first_root_element = 3 + anchor_gap + root_gap;
    s[first_root_element - 1] = tail.into();
    s[2 + anchor_gap] = guide.inc().into();
    Unit::from(s)
}

fn unalias_root(mut segment: Segment, anchor_gap: u32, root_gap: u32, count: u32, guide: Guide) -> Segment {
    let used_units = anchor_gap + root_gap + count + 3 /*anchor, prism, guide*/;
    let mut s = Segment::with_capacity(used_units);
    for i in 1..used_units {
        s[i] = segment[i];
    }
    for i in (used_units - count)..used_units {
        ValueUnit::from(s[i]).split()
    }
    if guide.has_meta() {
        // TODO
        // + 1 if external hash
        ValueUnit::from(s[3 + anchor_gap]).split()
    }
    if segment.unalias() == 0 {
        for i in (used_units - count)..used_units {
            ValueUnit::from(s[i]).retire()
        }
        if guide.has_meta() {
            // TODO
            // same as above
            ValueUnit::from(s[3 + anchor_gap]).retire()
        }
        Segment::free(segment)
    }
    s
}

fn conj_untailed_incomplete(prism: Line, x: Unit, guide: Guide, count: u32) -> Unit {
    let anchor_gap = guide.prism_to_anchor_gap();
    let segment: Segment = prism.offset(-((anchor_gap + 1) as isize)).into();
    if segment.is_aliased() {
        conj_untailed_incomplete_aliased(prism, x, guide, count, anchor_gap, segment)
    } else {
        conj_untailed_incomplete_unaliased(prism, x, guide, count, anchor_gap, segment)
    }
}

fn conj_untailed_incomplete_aliased(prism: Line, x: Unit, guide: Guide, count: u32,
                                    anchor_gap: u32, mut segment: Segment) -> Unit {
    let root_gap = guide.guide_to_root_gap();
    let used_units = anchor_gap + root_gap + count + 3 /*anchor, prism, guide*/;

    let new_count = (count + 1).next_power_of_two();
    let new_cap = used_units + (new_count - count);
    let (shift, guide) = if new_count == TAIL_CAP
        { (1, guide.inc_guide_to_root_gap()) } else { (0, guide) };
    let mut s = Segment::with_capacity(new_cap + shift);

    for i in 1..(used_units - count) {
        s[i] = segment[i]
    }
    for i in (used_units - count)..used_units {
        s[i + shift] = segment[i]
    }

    for i in (used_units - count)..used_units {
        ValueUnit::from(s[i + shift]).split()
    }
    if guide.has_meta() {
        ValueUnit::from(s[3 + anchor_gap]).split()
    }

    if segment.unalias() == 0 {
        for i in (used_units - count)..used_units {
            ValueUnit::from(s[i + shift]).retire()
        }
        if guide.has_meta() {
            ValueUnit::from(s[3 + anchor_gap]).retire()
        }
        Segment::free(segment)
    }

    s[used_units + shift] = x;
    s[2 + anchor_gap] = guide.inc().into();
    Unit::from(s)
}

fn conj_untailed_incomplete_unaliased(prism: Line, x: Unit, guide: Guide, count: u32,
                                      anchor_gap: u32, mut segment: Segment) -> Unit {
    let root_gap = guide.guide_to_root_gap();
    let used_units = anchor_gap + root_gap + count + 3 /*anchor, prism, guide*/;
    let cap = segment.capacity();

    if used_units == cap {
        let new_count = (count + 1).next_power_of_two();
        let new_cap = used_units + (new_count - count);
        let (shift, guide) = if new_count == TAIL_CAP
            { (1, guide.inc_guide_to_root_gap()) } else { (0, guide) };
        let mut s = Segment::with_capacity(new_cap + shift);

        for i in 1..(used_units - count) {
            s[i] = segment[i]
        }
        for i in (used_units - count)..used_units {
            s[i + shift] = segment[i]
        }

        s[used_units + shift] = x;
        s[2 + anchor_gap] = guide.inc().into();
        Unit::from(s)
    } else {
        segment[used_units] = x;
        segment[2 + anchor_gap] = guide.inc().into();
        Unit::from(segment)
    }
}

fn conj_tailed(prism: Line, x: Unit, guide: Guide, count: u32) -> Unit {
    let tailoff = (count - 1) >> BITS << BITS;
    let tail_count = count - tailoff;
    if tail_count != TAIL_CAP {
        let anchor_gap = guide.prism_to_anchor_gap();
        let segment: Segment = prism.offset(-((anchor_gap + 1) as isize)).into();
        let mut s = if segment.is_aliased() {
            unalias_root(segment, guide.prism_to_anchor_gap(),
                         guide.guide_to_root_gap(), count, guide)
        } else { segment };
        let first_root_element = 3 + anchor_gap + guide.guide_to_root_gap();
        let mut tail = Segment::from(s[first_root_element - 1]);
        let mut t = if tail.is_aliased() {
            let mut t = Segment::new(TAIL_CAP);
            for i in 1..(tail_count + 1) {
                t[i] = tail[i];
            }
            for i in 1..(tail_count + 1) {
                ValueUnit::from(t[i]).split()
            }
            if tail.unalias() == 0 {
                for i in 1..(tail_count + 1) {
                    ValueUnit::from(t[i]).retire()
                }
                Segment::free(tail)
            }
            t
        } else { tail };
        t[tail_count + 1] = x;
        s[first_root_element - 1] = t.into();
        s[2 + anchor_gap] = guide.inc().into();
        Unit::from(s)
    } else {
        conj_tailed_complete(prism, x, guide, count, tailoff, tail_count)
    }
}

fn conj_tailed_complete(prism: Line, x: Unit, guide: Guide, count: u32,
                        tailoff: u32, tail_count: u32) -> Unit {
    unimplemented!()
}





fn tree_count(count: u32) -> u32 {
    (count - 1) & !MASK
}

fn significant_bits(x: u32) -> u8 {
    /*bits in a u32*/ 32 - x.leading_zeros() as u8
}

fn digit_count(x: u32) -> u8 {
    ((significant_bits(x) as u32 + BITS - 1) as u32 / BITS) as u8
}

fn digit(x: u32, idx: u8) -> u8 {
    (x >> (idx as u32 * BITS)) as u8
}

fn digit_iter(x: u32, digits: u8) {
    // Digit iterator struct
}