// Copyright (c) Cole Frederick. All rights reserved.
// The use and distribution terms for this software are covered by the
// Eclipse Public License 1.0 (https://opensource.org/licenses/eclipse-1.0.php)
// which can be found in the file epl-v10.html at the root of this distribution.
// By using this software in any fashion, you are agreeing to be bound by the terms of this license.
// You must not remove this notice, or any other, from this software.

use super::*;

pub fn conj(prism: AnchoredLine, x: Unit) -> Unit {
    let guide= unaliased_root(Guide::hydrate(prism));
    if guide.count <= TAIL_CAP {
        conj_untailed(guide, x)
    } else {
        conj_tailed(guide, x)
    }
}

pub fn unalias_root(guide: Guide) -> Guide {
    if guide.count <= TAIL_CAP { // untailed
        let width = size(guide.count);
        let grew_tail_bit = guide.is_compact_bit & is_arity_bit(width);
        let g = {
            let s = Segment::new(guide.root.index + (width | grew_tail_bit));
            let mut g = guide;
            g.prism = guide.prism.with_seg(s);
            g.is_compact_bit = g.is_compact_bit & !grew_tail_bit;
            g.reroot()
        };
        guide.segment().at(0..guide.root.index).to(g.segment());
        let roots = guide.root.span(guide.count);
        roots.to_offset(g.segment(), g.root.index);
        guide.split_meta();
        roots.split();
        if guide.segment().unalias() == 0 {
            guide.retire_meta();
            roots.retire();
            Segment::free(guide.segment());
        }
        g
    } else { // tailed
        let root_count = root_content_count(tailoff(guide.count));
        let width = size(root_count + 1 /*tail*/);
        let g = {
            let cap = guide.root.index - 1 /*tail*/ + (width | is_arity_bit(width));
            let s = Segment::new(cap);
            let mut g = guide;
            g.prism = guide.prism.with_seg(s);
            g.reroot()
        };
        guide.segment().at(0..(guide.root.index + root_count)).to(g.segment());
        guide.split_meta();
        let tail_and_roots = guide.root.offset(-1).span(root_count + 1);
        tail_and_roots.alias();
        if guide.segment().unalias() == 0 {
            guide.retire_meta();
            tail_and_roots.unalias();
            Segment::free(guide.segment());
        }
        g
    }
}

pub fn unaliased_root(guide: Guide) -> Guide {
    if guide.segment().is_aliased() {
        unalias_root(guide)
    } else {
        guide
    }
}

pub fn conj_untailed(guide: Guide, x: Unit) -> Unit {
    if guide.count == TAIL_CAP { // complete
        let tail = {
            let tail = Segment::new(TAIL_CAP);
            tail.set(0, x);
            tail
        };
        guide.root.set(-1, tail.unit());
        guide.inc_count().store().segment().unit()
    } else { // incomplete
        if guide.root.has_index(guide.count as i32) {
            guide.root.set(guide.count as i32, x);
            guide.inc_count().store().segment().unit()
        } else {
            let width = size(guide.count);
            let grew_tail_bit = guide.is_compact_bit & is_arity_bit(width);
            let g = {
                let s = Segment::new(guide.root.index + (width | grew_tail_bit));
                let mut g = guide;
                g.prism = guide.prism.with_seg(s);
                g.is_compact_bit = g.is_compact_bit & !grew_tail_bit;
                g.reroot()
            };
            guide.segment().at(0..guide.root.index).to(g.segment());
            guide.root.span(guide.count).to_offset(g.segment(), g.root.index);
            Segment::free(guide.segment());
            g.root.set(g.count as i32, x);
            g.inc_count().store().segment().unit()
        }
    }
}

pub fn conj_tailed(guide: Guide, x: Unit) -> Unit {
    let tail_count = tail_count(guide.count);
    if tail_count != TAIL_CAP {
        let tail = guide.root[-1].segment();
        if tail.is_aliased() {
            let s = Segment::new(TAIL_CAP);
            let tails = tail.at(0..tail_count);
            tails.to(s);
            tails.split();
            if tail.unalias() == 0 {
                tails.retire();
                Segment::free(tail);
            }
            s.set(tail_count, x);
            guide.root.set(-1, s.unit());
            guide.inc_count().store().segment().unit()
        } else {
            tail.set(tail_count, x);
            guide.inc_count().store().segment().unit()
        }
    } else {
        conj_tailed_complete(guide, x)
    }
}

pub fn conj_tailed_complete(guide: Guide, x: Unit) -> Unit {
    let tailoff = guide.count - TAIL_CAP;
    let last_index = tailoff - 1;
    let path_diff = tailoff ^ last_index;
    use std::cmp::Ordering;
    match digit_count(last_index).cmp(&digit_count(path_diff)) {
        Ordering::Less    => { growing_height(guide, x, tailoff) },
        Ordering::Equal   => { growing_root(guide, x, tailoff) },
        Ordering::Greater => { growing_child(guide, x, tailoff) },
    }
}

pub fn path_of_height(height: u32, mut end: Unit) -> Unit {
    for _ in 0..height {
        let c = Segment::new(size(1) /*4*/);
        c.set(0, end);
        end = c.unit();
    }
    end
}

pub fn growing_height(guide: Guide, x: Unit, tailoff: u32) -> Unit {
    let g = {
        let s = {
            let s = Segment::new(guide.root.index + size(3) /*4*/);
            guide.segment().at(0..guide.root.index).to(s);
            let child = {
                let c = Segment::new(ARITY);
                guide.root.span(ARITY).to_offset(c, 0);
                c
            };
            s.set(guide.root.index, child.unit());
            s
        };
        let mut g = guide;
        g.prism = guide.prism.with_seg(s);
        Segment::free(guide.segment());
        g.reroot()
    };
    let path = path_of_height(trailing_zero_digit_count(tailoff >> BITS), g.root[-1]);
    g.root.set(1, path);
    let tail = {
        let t = Segment::new(TAIL_CAP);
        t.set(0, x);
        t
    };
    g.root.set(-1, tail.unit());
    g.inc_count().store().segment().unit()
}

pub fn growing_root(guide: Guide, x: Unit, tailoff: u32) -> Unit {
    let root_count = root_content_count(tailoff);
    let g = if guide.root.has_index(root_count as i32) {
        guide
    } else {
        let g = {
            let grown_root_count = root_count + 1;
            let width = size(grown_root_count + 1 /*tail*/);
            let cap = guide.root.index - 1 /*tail*/ + (width | is_arity_bit(width));
            let s = Segment::new(cap);
            let mut g = guide;
            g.prism = guide.prism.with_seg(s);
            g.reroot()
        };
        guide.segment().at(0..(guide.root.index + root_count)).to(g.segment());
        Segment::free(guide.segment());
        g
    };
    let path = path_of_height(trailing_zero_digit_count(tailoff >> BITS), g.root[-1]);
    g.root.set(root_count as i32, path);
    let tail = {
        let t = Segment::new(TAIL_CAP);
        t.set(0, x);
        t
    };
    g.root.set(-1, tail.unit());
    g.inc_count().store().segment().unit()
}


pub fn create_path(root: AnchoredLine, path: u32, height: u32, path_length: u32) -> AnchoredLine {
    let mut shift = (height - 1) * BITS;
    let mut curr = {
        let ret = root.offset(last_digit(path >> shift) as i32);
        shift -= BITS;
        ret
    };
    for _ in 1..path_length {
        let s = curr[0].segment();
        let digit = {
            let ret = last_digit(path >> shift);
            shift -= BITS;
            ret
        };
        if !s.is_aliased() {
            if s.has_index(digit) {
                curr = s.line_at(digit);
            } else {
                let t = Segment::new(size(digit + 1));
                s.at(0..(digit + 1)).to(t);
                Segment::free(s);
                curr.set(0, t.unit());
                curr = t.line_at(digit);
            }
        } else {
            let t = {
                let t = Segment::new(size(digit + 1));
                let range = s.at(0..(digit + 1));
                range.to(t);
                range.alias();
                if s.unalias() == 0 {
                    range.unalias();
                    Segment::free(s);
                }
                t
            };
            curr.set(0, t.unit());
            curr = t.line_at(digit);
        }
    }
    curr
}

pub fn growing_child(guide: Guide, x: Unit, tailoff: u32) -> Unit {
    let zero_count = trailing_zero_digit_count(tailoff);
    let digit_count = digit_count(tailoff);
    let c = create_path(guide.root, tailoff, digit_count, digit_count - zero_count);
    let path = path_of_height(zero_count - 1, guide.root[-1]);
    c.set(0, path);
    let tail = {
        let t = Segment::new(TAIL_CAP);
        t.set(0, x);
        t
    };
    guide.root.set(-1, tail.unit());
    guide.inc_count().store().segment().unit()
}
