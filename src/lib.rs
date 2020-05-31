mod context_chain;
mod flower;
mod utils;

extern crate base64;
extern crate deflate;
extern crate if_chain;
extern crate inflate;
extern crate regex;

use context_chain::ContextChain;
use if_chain::if_chain;
use js_sys;
#[macro_use]
extern crate lazy_static;
use regex::Regex;
use std::cmp;
use std::f64;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// lifted from the `console_log` example
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);

    fn set_tweet_button(url: &str, text: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

use flower::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::rc::Rc;
// use Gene::{G00, G01, G11};

const IMG_SIZE: f64 = 50_f64;
const IMG_GRID_RATIO: f64 = 1.6;

#[derive(Debug)]
enum CanvasCell {
    FlowerCell { flower: Flower, elm: HtmlElement },
    HybridLineCell { hybrids: HashSet<HT> },
}

impl CanvasCell {
    fn new_flower(flower: Flower, document: &Document, img_size: f64) -> Result<Self, JsValue> {
        let grid_size = img_size * IMG_GRID_RATIO;
        let flower_div = document
            .create_element("div")?
            .dyn_into::<HtmlElement>()
            .unwrap();
        // let (x, y) = position;
        flower_div.set_class_name("flower-cell");
        flower_div.set_css_styles(vec![
            /*
            ("position", "absolute"),
            (
                "top",
                format!("{}px", grid_size * y as f64 - 3_f64).as_str(),
            ),
            (
                "left",
                format!("{}px", grid_size * x as f64 - 3_f64).as_str(),
            ),
             */
            ("height", format!("{}px", grid_size).as_str()),
            ("width", format!("{}px", grid_size).as_str()),
            // ("text-align", "center"),
            // ("padding-top", format!("{}px", grid_size / 10_f64).as_str()),
            // ("padding-top", format!("{}px", 0).as_str()),
        ])?;

        let img = document
            .create_element("img")?
            .dyn_into::<HtmlImageElement>()
            .unwrap();
        img.set_src(format!("/{}.png", flower.get_name()).as_str());
        img.set_css_styles(vec![
            ("display", "block"),
            ("height", format!("{}px", img_size).as_str()),
            ("margin", "auto"),
            ("margin-top", "0"),
            ("margin-bottom", "0"),
            // ("margin-top", format!("{}px", grid_size / 10_f64).as_str()),
            // ("height", format!("{}px", img_size).as_str()),
        ])?;

        flower_div.append_child(&img)?;

        let genome_label = flower
            .to_string()
            .split('-')
            .into_iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "<span style=\"border-bottom: {} 1px;\">{}</span>",
                    ["red solid", "yellow solid", "black dashed", "#BAD3FF solid"][i % 4],
                    s,
                )
            })
            .collect::<Vec<String>>()
            .join("-");

        let div = document
            .create_element("div")?
            .dyn_into::<HtmlElement>()
            .unwrap();
        div.set_css_styles(vec![
            ("font-size", "11px"),
            ("font-weight", "bold"),
            ("line-height", "1.0"),
        ])?;
        div.set_inner_html(
            format!(
                "{}<br />{}",
                flower.get_name().replace("_seed", ""),
                genome_label
            )
            .as_str(),
        );

        flower_div.append_child(&div)?;

        /*
        let br = document
            .create_element("br")?
            .dyn_into::<HtmlElement>()
            .unwrap();
        flower_div.append_child(&br)?;

        let span = document
            .create_element("span")?
            .dyn_into::<HtmlElement>()
            .unwrap();
        span.set_css_styles(vec![("font-size", "11px")])?;
        span.set_inner_html(flower.to_string().as_str());

        flower_div.append_child(&span)?;
         */

        Ok(CanvasCell::FlowerCell {
            flower,
            elm: flower_div,
        })
    }

    fn get_elm(&self) -> Option<&HtmlElement> {
        match self {
            CanvasCell::FlowerCell { flower: _, elm } => Some(&elm),
            _ => None,
        }
    }

    fn get_flower(&self) -> Option<&Flower> {
        match self {
            CanvasCell::FlowerCell { flower, elm: _ } => Some(&flower),
            _ => None,
        }
    }

    fn new_hybrid() -> Self {
        CanvasCell::HybridLineCell {
            hybrids: HashSet::new(),
        }
    }

    fn remove_hybrid(&mut self, hybrid_tuple: &HT) -> bool {
        if let Self::HybridLineCell { hybrids } = self {
            hybrids.remove(hybrid_tuple);
            return hybrids.is_empty();
        }
        false
    }

    fn is_flower_cell(&self) -> bool {
        if let Self::FlowerCell { flower: _, elm: _ } = self {
            true
        } else {
            false
        }
    }

    fn is_hybrid_cell(&self) -> bool {
        if let Self::HybridLineCell { hybrids: _ } = self {
            true
        } else {
            false
        }
    }
}

impl Drop for CanvasCell {
    fn drop(&mut self) {
        match self {
            Self::FlowerCell { flower: _, elm } => elm.remove(),
            _ => (),
        }
    }
}

trait ExElm {
    fn set_css_styles(&self, properties: Vec<(&str, &str)>) -> Result<(), JsValue>;
}

impl ExElm for HtmlElement {
    fn set_css_styles(&self, properties: Vec<(&str, &str)>) -> Result<(), JsValue> {
        for (p, v) in properties {
            self.style().set_property(p, v)?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Dir {
    North,
    East,
    South,
    West,
}

use Dir::*;

impl Dir {
    fn decide_dir(
        f: (usize, usize),
        m: (usize, usize),
        of: (usize, usize),
    ) -> Option<(Self, bool)> {
        let mut is_short = false;

        let right = if cmp::max(f.0, m.0) < of.0 {
            is_short = of.0 - cmp::max(f.0, m.0) <= 1;
            if !is_short {
                f.1 != m.1
            } else {
                let max_y = cmp::max(f.1, m.1);
                let min_y = cmp::min(f.1, m.1);
                // max_y - min_y > 1 &&
                min_y < of.1 && of.1 < max_y
            }
        } else {
            false
        };

        if right {
            return Some((East, is_short));
        }

        let down = if cmp::max(f.1, m.1) < of.1 {
            is_short = of.1 - cmp::max(f.1, m.1) <= 1;
            if !is_short {
                f.0 != m.0
            } else {
                let max_x = cmp::max(f.0, m.0);
                let min_x = cmp::min(f.0, m.0);
                // max_x - min_x > 1 &&
                min_x < of.0 && of.0 < max_x
            }
        } else {
            false
        };

        if down {
            return Some((South, is_short));
        }

        let left = if of.0 < cmp::min(f.0, m.0) {
            is_short = cmp::min(f.0, m.0) - of.0 <= 1;
            if !is_short {
                f.1 != m.1
            } else {
                let max_y = cmp::max(f.1, m.1);
                let min_y = cmp::min(f.1, m.1);
                // max_y - min_y > 1 &&
                min_y < of.1 && of.1 < max_y
            }
        } else {
            false
        };

        if left {
            return Some((West, is_short));
        }

        let up = if of.1 < cmp::min(f.1, m.1) {
            is_short = cmp::min(f.1, m.1) - of.1 <= 1;
            if !is_short {
                f.0 != m.0
            } else {
                let max_x = cmp::max(f.0, m.0);
                let min_x = cmp::min(f.0, m.0);
                // max_x - min_x > 1 &&
                min_x < of.0 && of.0 < max_x
            }
        } else {
            false
        };

        if up {
            return Some((North, is_short));
        }

        None
    }
}

use std::cmp::{Eq, PartialEq};

// Hybrid Tuple
#[derive(Hash, Clone, Copy, Debug)]
struct HT((usize, usize), (usize, usize), (usize, usize));

impl PartialEq for HT {
    fn eq(&self, other: &Self) -> bool {
        ((self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0))
            && self.2 == other.2
    }
}

impl Eq for HT {}

impl fmt::Display for HT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?}, {:?}, {:?})", self.0, self.1, self.2,)
    }
}

struct HybridLine {
    ht: HT,
    occupied_pos: Vec<(usize, usize)>,
    draw_line: Box<dyn Fn(&CanvasRenderingContext2d, f64)>,
    draw_chance: Box<dyn Fn(&CanvasRenderingContext2d, f64, &CellBook)>,
}

use std::fmt;

impl fmt::Debug for HybridLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridLine").field("y", &self.ht).finish()
    }
}

impl HybridLine {
    fn new(ht: HT, cell_book: &mut CellBook) -> Result<Self, JsValue> {
        let HT(f, m, of) = ht;
        let (dir, is_short) =
            Dir::decide_dir(f, m, of).ok_or(JsValue::from("Can't decide hybrid direction."))?;
        // let ht = HT(f, m, of);
        let mut occupied_pos = Vec::new();

        fn hybrid_cell_insert(m: &mut CellBook, t: (usize, usize), ht: HT) -> Result<(), JsValue> {
            let cell = m.entry(t).or_insert(CanvasCell::new_hybrid());
            match cell {
                CanvasCell::HybridLineCell { hybrids } => {
                    hybrids.insert(ht);
                }
                _ => return Err(JsValue::from("There will be already occupied by a flower.")),
            }

            Ok(())
        }

        let center_line: Vec<(usize, usize)>;
        let center_begin: (usize, usize);
        let center_end: (usize, usize);
        let father_line: Vec<(usize, usize)>;
        let father_end: Option<(usize, usize)>;
        let mother_line: Vec<(usize, usize)>;
        let mother_end: Option<(usize, usize)>;
        let offspring_line: Vec<(usize, usize)>;
        let offspring_end: (usize, usize);
        let percentage_pos: (usize, usize);

        let dif = match dir {
            North | West => -1,
            South | East => 1,
        };

        match dir {
            North | South => {
                let front = if dir == North {
                    cmp::min(f.1, m.1)
                } else {
                    cmp::max(f.1, m.1)
                };
                if !is_short {
                    let axis = (front + of.1) / 2;
                    let x1 = cmp::min(cmp::min(f.0, m.0), of.0);
                    let x2 = cmp::max(cmp::max(f.0, m.0), of.0);
                    center_line = (x1..=x2).map(|x| (x, axis)).collect::<Vec<_>>();
                    center_begin = (x1, axis);
                    center_end = (x2, axis);
                    let axis_p = ((axis as i32) - dif) as usize;
                    let f_dy = ((f.1 as i32) + dif) as usize;
                    father_line = (cmp::min(f_dy, axis_p)..=cmp::max(f_dy, axis_p))
                        .map(|y| (f.0, y))
                        .collect::<Vec<_>>();
                    father_end = Some((f.0, axis));
                    let m_dy = ((m.1 as i32) + dif) as usize;
                    mother_line = (cmp::min(m_dy, axis_p)..=cmp::max(m_dy, axis_p))
                        .map(|y| (m.0, y))
                        .collect::<Vec<_>>();
                    mother_end = Some((m.0, axis));
                    let axis_o = ((axis as i32) + dif) as usize;
                    let of_dy = ((of.1 as i32) - dif) as usize;
                    offspring_line = (cmp::min(of_dy, axis_o)..=cmp::max(of_dy, axis_o))
                        .map(|y| (of.0, y))
                        .collect::<Vec<_>>();
                    offspring_end = (of.0, axis);
                    percentage_pos = (of.0, of_dy);
                } else {
                    let axis = front;
                    let x1 = cmp::min(f.0, m.0);
                    let x2 = cmp::max(f.0, m.0);
                    center_line = (x1..=x2).map(|x| (x, axis)).collect::<Vec<_>>();
                    center_begin = (x1, axis);
                    center_end = (x2, axis);
                    let axis_p = ((axis as i32) - dif) as usize;
                    if front == f.1 {
                        father_line = Vec::new();
                        father_end = None;
                    } else {
                        let f_dy = ((f.1 as i32) + dif) as usize;
                        father_line = (cmp::min(f_dy, axis_p)..=cmp::max(f_dy, axis_p))
                            .map(|y| (f.0, y))
                            .collect::<Vec<_>>();
                        father_end = Some((f.0, axis));
                    }
                    if front == m.1 {
                        mother_line = Vec::new();
                        mother_end = None;
                    } else {
                        let m_dy = ((m.1 as i32) + dif) as usize;
                        mother_line = (cmp::min(m_dy, axis_p)..=cmp::max(m_dy, axis_p))
                            .map(|y| (m.0, y))
                            .collect::<Vec<_>>();
                        mother_end = Some((m.0, axis));
                    }
                    offspring_line = Vec::new();
                    offspring_end = (of.0, axis);
                    percentage_pos = (of.0, axis);
                }
            }
            East | West => {
                let front = if dir == West {
                    cmp::min(f.0, m.0)
                } else {
                    cmp::max(f.0, m.0)
                };
                if !is_short {
                    let axis = (front + of.0) / 2;
                    let y1 = cmp::min(cmp::min(f.1, m.1), of.1);
                    let y2 = cmp::max(cmp::max(f.1, m.1), of.1);
                    center_line = (y1..=y2).map(|y| (axis, y)).collect::<Vec<_>>();
                    center_begin = (axis, y1);
                    center_end = (axis, y2);
                    let axis_p = ((axis as i32) - dif) as usize;
                    let f_dx = ((f.0 as i32) + dif) as usize;
                    father_line = (cmp::min(f_dx, axis_p)..=cmp::max(f_dx, axis_p))
                        .map(|x| (x, f.1))
                        .collect::<Vec<_>>();
                    father_end = Some((axis, f.1));
                    let m_dx = ((m.0 as i32) + dif) as usize;
                    mother_line = (cmp::min(m_dx, axis_p)..=cmp::max(m_dx, axis_p))
                        .map(|x| (x, m.1))
                        .collect::<Vec<_>>();
                    mother_end = Some((axis, m.1));
                    let axis_o = ((axis as i32) + dif) as usize;
                    let of_dx = ((of.0 as i32) - dif) as usize;
                    offspring_line = (cmp::min(of_dx, axis_o)..=cmp::max(of_dx, axis_o))
                        .map(|x| (x, of.1))
                        .collect::<Vec<_>>();
                    offspring_end = (axis, of.1);
                    percentage_pos = (of_dx, of.1);
                } else {
                    let axis = front;
                    let y1 = cmp::min(f.1, m.1);
                    let y2 = cmp::max(f.1, m.1);
                    center_line = (y1..=y2).map(|y| (axis, y)).collect::<Vec<_>>();
                    center_begin = (axis, y1);
                    center_end = (axis, y2);
                    let axis_p = ((axis as i32) - dif) as usize;
                    if front == f.0 {
                        father_line = Vec::new();
                        father_end = None;
                    } else {
                        let f_dx = ((f.0 as i32) + dif) as usize;
                        father_line = (cmp::min(f_dx, axis_p)..=cmp::max(f_dx, axis_p))
                            .map(|x| (x, f.1))
                            .collect::<Vec<_>>();
                        father_end = Some((axis, f.1));
                    }
                    if front == m.0 {
                        mother_line = Vec::new();
                        mother_end = None;
                    } else {
                        let m_dx = ((m.0 as i32) + dif) as usize;
                        mother_line = (cmp::min(m_dx, axis_p)..=cmp::max(m_dx, axis_p))
                            .map(|x| (x, m.1))
                            .collect::<Vec<_>>();
                        mother_end = Some((axis, m.1));
                    }
                    offspring_line = Vec::new();
                    offspring_end = (axis, of.1);
                    percentage_pos = (axis, of.1);
                }
            }
        };

        for line in vec![center_line, father_line, mother_line, offspring_line] {
            for (x, y) in line {
                let t = (x, y);
                if t == f || t == m || t == of {
                    continue;
                }
                occupied_pos.push(t);
                hybrid_cell_insert(cell_book, t, ht)?;
            }
        }

        fn grid_center(pos: (usize, usize), g: f64) -> (f64, f64) {
            ((pos.0 as f64 + 0.5) * g, (pos.1 as f64 + 0.5) * g)
        }

        let draw_line: Box<dyn Fn(&CanvasRenderingContext2d, f64)> =
            Box::new(move |ctx, grid_size| {
                let g4 = grid_size / 4_f64;
                let g8 = grid_size / 8_f64;

                let cb = grid_center(center_begin, grid_size);
                let ce = grid_center(center_end, grid_size);
                ContextChain::new(ctx)
                    .set_stroke_style(&JsValue::from("#0000FF"))
                    .set_line_width(3_f64)
                    .begin_path()
                    .move_to(cb.0, cb.1)
                    .line_to(ce.0, ce.1)
                    .stroke();

                for (b, end) in vec![(f, father_end), (m, mother_end)] {
                    if let Some(e) = end {
                        let b = grid_center(b, grid_size);
                        let e = grid_center(e, grid_size);
                        ContextChain::new(ctx)
                            .begin_path()
                            .set_line_width(3_f64)
                            .move_to(b.0, b.1)
                            .line_to(e.0, e.1)
                            .stroke();

                        let ar1;
                        let ar2;
                        let ar3;
                        if b.0 == e.0 {
                            let t = if b.1 < e.1 { g4 } else { -g4 };
                            ar1 = (b.0 - g8, b.1 + t);
                            ar2 = (b.0 + g8, b.1 + t);
                            ar3 = (b.0, b.1 + t * 2_f64);
                        } else {
                            let t = if b.0 < e.0 { g4 } else { -g4 };
                            ar1 = (b.0 + t, b.1 - g8);
                            ar2 = (b.0 + t, b.1 + g8);
                            ar3 = (b.0 + t * 2_f64, b.1);
                        }

                        ContextChain::new(&ctx)
                            .begin_path()
                            .set_line_width(1_f64)
                            .move_to(ar1.0, ar1.1)
                            .line_to(ar2.0, ar2.1)
                            .line_to(ar3.0, ar3.1)
                            .line_to(ar1.0, ar1.1)
                            .stroke();
                        ctx.set_fill_style(&JsValue::from("#0000FF"));
                        ctx.fill();
                    }
                }

                let b = grid_center(of, grid_size);
                let e = grid_center(offspring_end, grid_size);
                ContextChain::new(ctx)
                    .begin_path()
                    .set_line_width(3_f64)
                    .move_to(b.0, b.1)
                    .line_to(e.0, e.1)
                    .stroke();

                let ar1;
                let ar2;
                let ar3;
                if b.0 == e.0 {
                    let t = if b.1 < e.1 { g4 } else { -g4 };
                    ar1 = (b.0 + g8, b.1 + t * 2_f64);
                    ar2 = (b.0 - g8, b.1 + t * 2_f64);
                    ar3 = (b.0, b.1 + t);
                } else {
                    let t = if b.0 < e.0 { g4 } else { -g4 };
                    ar1 = (b.0 + t * 2_f64, b.1 + g8);
                    ar2 = (b.0 + t * 2_f64, b.1 - g8);
                    ar3 = (b.0 + t, b.1);
                }

                ContextChain::new(&ctx)
                    .begin_path()
                    .set_line_width(1_f64)
                    .move_to(ar1.0, ar1.1)
                    .line_to(ar2.0, ar2.1)
                    .line_to(ar3.0, ar3.1)
                    .line_to(ar1.0, ar1.1)
                    .stroke();
                ctx.set_fill_style(&JsValue::from("#0000FF"));
                ctx.fill();
            });

        let draw_chance: Box<dyn Fn(&CanvasRenderingContext2d, f64, &CellBook)> =
            Box::new(move |ctx, grid_size, cb| {
                // Show percentage
                let father_flower_w = cb.get(&f).and_then(|v| match v {
                    CanvasCell::FlowerCell { flower, elm: _ } => Some(flower),
                    _ => None,
                });

                let mother_flower_w = cb.get(&m).and_then(|v| match v {
                    CanvasCell::FlowerCell { flower, elm: _ } => Some(flower),
                    _ => None,
                });

                let offspring_flower_w = cb.get(&of).and_then(|v| match v {
                    CanvasCell::FlowerCell { flower, elm: _ } => Some(flower),
                    _ => None,
                });

                if_chain! {
                    if let Some(f_f) = father_flower_w;
                    if let Some(m_f) = mother_flower_w;
                    if let Some(of_f) = offspring_flower_w;
                    then {
                        let ppos = (
                            percentage_pos.0 as f64 * grid_size,
                            percentage_pos.1 as f64 * grid_size,
                        );
                        let cpos = grid_center(percentage_pos, grid_size);
                        let p = (of_f.get_chance_from(f_f, m_f) as f64) / 256_f64 * 100_f64;
                        let g2 = grid_size / 2_f64;
                        let g4 = grid_size / 4_f64;
                        let g8 = grid_size / 8_f64;
                        ContextChain::new(ctx)
                            .set_fill_style(&JsValue::from("#FFFFFF"))
                            .fill_rect(ppos.0 + g8, cpos.1 - g8, g2 + g4, g4)
                            .set_text_baseline("middle")
                            .set_text_align("center")
                            .set_fill_style(&JsValue::from("#0000FF"))
                            .set_font(format!("{}px sans-serif", grid_size * 0.2).as_str())
                            .fill_text(format!("{:.1}%", p).as_str(), cpos.0, cpos.1)
                            .stroke();

                    }
                }
            });

        Ok(HybridLine {
            ht,
            occupied_pos,
            draw_line,
            draw_chance,
        })
    }
}

/*
impl Drop for HybridLine {
    fn drop(&mut self) {
        for p in self.occupied_pos.iter() {
            // console_log!("[433] beep");
            let mut book = self.cell_book.borrow_mut();
            // console_log!("[535] beep");
            if let Some(cell) = book.get_mut(p) {
                let res = cell.remove_hybrid(&self.ht);
                if res {
                    book.remove(p);
                }
            }
        }
    }
}
 */

type CellBook = HashMap<(usize, usize), CanvasCell>;
type HybridBook = HashMap<HT, HybridLine>;

struct CanvasBook {
    inp_filename: HtmlInputElement,
    cell_book: CellBook,
    hybrid_book: HybridBook,
}

impl fmt::Display for CanvasBook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} | {:?}", self.cell_book, self.hybrid_book)
    }
}

impl CanvasBook {
    fn new(inp_filename: HtmlInputElement, cell_book: CellBook, hybrid_book: HybridBook) -> Self {
        CanvasBook {
            inp_filename,
            cell_book,
            hybrid_book,
        }
    }

    fn c_insert(&mut self, key: (usize, usize), item: CanvasCell) -> Option<CanvasCell> {
        let s = self.cell_book.insert(key, item);
        if let Err(v) = self.save() {
            console_log!("Caution!: Save was failed. {:?}", v);
        }
        s
    }

    fn c_remove(&mut self, key: &(usize, usize)) -> Option<CanvasCell> {
        let s = self.cell_book.remove(key);
        if let Err(v) = self.save() {
            console_log!("Caution!: Save was failed. {:?}", v);
        }
        s
    }

    fn c_get(&self, key: &(usize, usize)) -> Option<&CanvasCell> {
        self.cell_book.get(key)
    }

    fn c_get_mut(&mut self, key: &(usize, usize)) -> Option<&mut CanvasCell> {
        self.cell_book.get_mut(key)
    }

    fn c_iter(&self) -> std::collections::hash_map::Iter<(usize, usize), CanvasCell> {
        self.cell_book.iter()
    }

    fn _c_entry(
        &mut self,
        key: (usize, usize),
    ) -> std::collections::hash_map::Entry<(usize, usize), CanvasCell> {
        self.cell_book.entry(key)
    }

    fn h_insert(&mut self, key: HT, item: HybridLine) -> Option<HybridLine> {
        let s = self.hybrid_book.insert(key, item);
        if let Err(v) = self.save() {
            console_log!("Caution!: Save was failed. {:?}", v);
        }
        s
    }

    fn h_remove(&mut self, key: &HT) -> Option<HybridLine> {
        let s = self.hybrid_book.remove(key);
        if let Err(v) = self.save() {
            console_log!("Caution!: Save was failed. {:?}", v);
        }
        s
    }

    fn h_get(&self, key: &HT) -> Option<&HybridLine> {
        self.hybrid_book.get(key)
    }

    fn _h_get_mut(&mut self, key: &HT) -> Option<&mut HybridLine> {
        self.hybrid_book.get_mut(key)
    }

    fn h_iter(&self) -> std::collections::hash_map::Iter<'_, HT, HybridLine> {
        self.hybrid_book.iter()
    }

    fn make_genome_shorter(s: String) -> String {
        s.split('-')
            .map(|t| match t {
                "00" => '0',
                "01" => '1',
                _ => '2',
            })
            .collect::<String>()
    }

    fn get_hash(&self) -> String {
        let filename = self.inp_filename.value().replace("|", "");

        lazy_static! {
            static ref RE: Regex = Regex::new(r"[()\s]").unwrap();
        }
        let flowers_data = self
            .cell_book
            .iter()
            .filter(|(_, cell)| cell.is_flower_cell())
            .map(|(key, cell)| {
                let flower = cell.get_flower().unwrap();
                format!(
                    "{}@{:?}:{}",
                    flower.kind.to_string(),
                    key,
                    Self::make_genome_shorter(flower.to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("/");
        let flowers_data = RE.replace_all(flowers_data.as_str(), "");
        let hybrids_data = self
            .hybrid_book
            .keys()
            .map(HT::to_string)
            .collect::<Vec<_>>()
            .join("/");
        let hybrids_data = RE.replace_all(hybrids_data.as_str(), "");
        let data = format!("{}|{}|{}", filename, flowers_data, hybrids_data);
        // console_log!("raw: {} / {}", data.len(), data);
        // let b64 = base64::encode(data.clone());
        // console_log!("raw_base64 {} / {}", b64.len(), b64);
        let dflt = deflate::deflate_bytes(data.as_bytes());
        // console_log!("deflate: {} / {:?}", dflt.len(), dflt);
        let b64 = base64::encode(dflt);
        // console_log!("def base64: {} / {}", b64.len(), b64);

        b64
    }

    fn load(sing: &Singleton, raw_code: &str) -> Result<Self, JsValue> {
        let d64 = base64::decode(raw_code.as_bytes())
            .map_err(|err| JsValue::from(err.to_string().as_str()))?;
        // console_log!("d64: {} / {:?}", d64.len(), d64);
        let iflt = inflate::inflate_bytes(&d64).map_err(|err| JsValue::from(err.as_str()))?;
        // console_log!("iflt: {} / {:?}", iflt.len(), iflt);
        let data = String::from_utf8(iflt).map_err(|e| JsValue::from(format!("{:?}", e)))?;
        // console_log!("decode: {} / {}", data.len(), data);

        let mut sp = data.split('|');
        let filename = sp
            .next()
            .ok_or(JsValue::from("LOAD: filename nothing."))?
            .to_string();
        let flower_part = sp
            .next()
            .ok_or(JsValue::from("LOAD: flower part nothing."))?;
        let hybrid_part = sp
            .next()
            .ok_or(JsValue::from("LOAD: hybrid part nothing."))?;

        lazy_static! {
            static ref FLOWER_RE: Regex = Regex::new(r"^([a-z]+)@(\d+),(\d+):(\d{4})$").unwrap();
        }

        let mut cell_book: CellBook = if flower_part.len() > 0 {
            flower_part
                .split('/')
                .map(|s| {
                    let er = || -> JsValue { JsValue::from("LOAD: invalid flower code.") };
                    let caps = FLOWER_RE.captures(s).ok_or(er())?;
                    // 正規表現により整数であることは保証...しかしオーバーフローの可能性を考えResultに
                    let x = caps
                        .get(2)
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .map_err(|_| er())?;
                    let y = caps
                        .get(3)
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .map_err(|_| er())?;
                    let flower = Flower::strs2flower(
                        caps.get(1).unwrap().as_str(),
                        caps.get(4).unwrap().as_str(),
                    )
                    .ok_or(er())?;
                    let flower_cell =
                        CanvasCell::new_flower(flower, &sing.document, sing.img_size)?;
                    Ok(((x, y), flower_cell))
                })
                .collect::<Result<CellBook, JsValue>>()?
        } else {
            HashMap::new()
        };

        lazy_static! {
            static ref HYBRID_RE: Regex =
                Regex::new(r"^(\d+),(\d+),(\d+),(\d+),(\d+),(\d+)$").unwrap();
        }

        let hybrid_book: HybridBook = if hybrid_part.len() > 0 {
            hybrid_part
                .split('/')
                .map(|s| {
                    let er = || -> JsValue { JsValue::from("LOAD: invalid hybrid code.") };
                    let caps = HYBRID_RE.captures(s).ok_or(er())?;
                    let v = (1..7)
                        .map(|i| {
                            caps.get(i)
                                .unwrap()
                                .as_str()
                                .parse::<usize>()
                                .map_err(|_| er())
                        })
                        .collect::<Result<Vec<_>, JsValue>>()?;
                    let ht = HT((v[0], v[1]), (v[2], v[3]), (v[4], v[5]));
                    let hybrid_line = HybridLine::new(ht, &mut cell_book).map_err(|_| er())?;
                    Ok((ht, hybrid_line))
                })
                .collect::<Result<HybridBook, JsValue>>()?
        } else {
            HashMap::new()
        };

        let inp_filename = sing
            .document
            .get_element_by_id("filename")
            .expect("Error!: inp_filename")
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        inp_filename.set_value(filename.as_str());

        let cb = CanvasBook::new(inp_filename, cell_book, hybrid_book);
        Ok(cb)
    }

    fn save(&self) -> Result<(), JsValue> {
        if self.cell_book.is_empty() && self.hybrid_book.is_empty() {
            return Ok(());
        }

        let b64 = self.get_hash();

        window()
            .expect("Error!: window")
            .location()
            .set_hash(b64.as_str())?;

        /*
        let (cell_book, hybrid_book) = load(b64.as_str(), s)?;

        console_log!("cell_book: {:?}\nhybrid_book: {:?}", cell_book, hybrid_book);
         */

        Ok(())
    }

    fn plant_flower(&self, s: &Singleton) -> Result<(), JsValue> {
        self.cell_book
            .iter()
            .map(|(key, cell)| {
                if cell.is_flower_cell() {
                    let elm = cell.get_elm().unwrap();
                    render_flower(key.0, key.1, elm, &s)?;
                }
                Ok(())
            })
            .collect::<Result<(), JsValue>>()?;
        Ok(())
    }

    fn get_filename_from_hash(raw_code: &str) -> Result<String, JsValue> {
        let d64 = base64::decode(raw_code.as_bytes())
            .map_err(|err| JsValue::from(err.to_string().as_str()))?;
        // console_log!("d64: {} / {:?}", d64.len(), d64);
        let iflt = inflate::inflate_bytes(&d64).map_err(|err| JsValue::from(err.as_str()))?;
        // console_log!("iflt: {} / {:?}", iflt.len(), iflt);
        let data = String::from_utf8(iflt).map_err(|e| JsValue::from(format!("{:?}", e)))?;
        // console_log!("decode: {} / {}", data.len(), data);

        let mut sp = data.split('|');
        let filename = sp
            .next()
            .ok_or(JsValue::from("LOAD: filename nothing."))?
            .to_string();
        Ok(filename)
    }
}

struct GrabedImg(HtmlImageElement);

impl Drop for GrabedImg {
    fn drop(&mut self) {
        self.0.remove();
    }
}

struct Singleton {
    window: Window,
    document: Document,
    editor: HtmlElement,
    img_size: f64,
    grid_size: f64,
    contextmenu: HtmlElement,
    // contextmenu_is_popuped: bool,
    contextmenu_pos: Option<(usize, usize)>,
    contextmenu_candidate_flowers: Vec<CanvasCell>,
    hybrid_from_index: Option<usize>,
    canvas: HtmlCanvasElement,
    /*
    cell_book: CellBook,
    hybrid_book: HybridBook,
     */
    canvas_book: CanvasBook,
    inp_filename: HtmlInputElement,
    inp_kind: HtmlSelectElement,
    inp_genome: HtmlInputElement,
    selected_seed_cell: Option<CanvasCell>,
    selected_seed_elm: HtmlElement,
    father: Option<(usize, usize)>,
    father_cell: Option<CanvasCell>,
    father_elm: HtmlElement,
    mother: Option<(usize, usize)>,
    mother_cell: Option<CanvasCell>,
    mother_elm: HtmlElement,
    inp_mode_menu: HtmlInputElement,
    inp_mode_seed: HtmlInputElement,
    inp_mode_hybrid: HtmlInputElement,
    inp_mode_remove: HtmlInputElement,
    /*
    father_img: HtmlImageElement,
    father_str: HtmlElement,
    mother_img: HtmlImageElement,
    mother_str: HtmlElement,
    */
    mousedown_startpos: (usize, usize),
    grabed_img: Option<GrabedImg>,
    pallet: HtmlElement,
    pallet_inp_color: HtmlSelectElement,
    overlay: HtmlElement,
    // modal: HtmlElement,
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    utils::set_panic_hook();

    let window = window().expect("Error!: window");
    let document = window.document().expect("Error!: document");
    let editor = document
        .get_element_by_id("editor")
        .expect("Error!: editor")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let contextmenu = document
        .get_element_by_id("contextmenu")
        .expect("Error!: contextmenu")
        .dyn_into::<HtmlElement>()
        .unwrap();
    // let contextmenu_is_popuped = false;
    let contextmenu_pos = None;
    let contextmenu_candidate_flowers = Vec::new();
    let hybrid_from_index = None;
    let canvas = document
        .get_element_by_id("canvas")
        .expect("Error!: canvas")
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let cell_book = HashMap::new();
    let hybrid_book = HashMap::new();
    let inp_filename = document
        .get_element_by_id("filename")
        .expect("Error!: inp_filename")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let canvas_book = CanvasBook::new(inp_filename, cell_book, hybrid_book);
    let inp_filename = document
        .get_element_by_id("filename")
        .expect("Error!: inp_filename")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let inp_kind = document
        .get_element_by_id("seed-flower-kind")
        .expect("Error!: inp_kind")
        .dyn_into::<HtmlSelectElement>()
        .unwrap();
    let inp_genome = document
        .get_element_by_id("seed-genome")
        .expect("Error!: inp_genome")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let selected_seed_cell = None;
    let selected_seed_elm = document
        .get_element_by_id("selected-seed")
        .expect("Error!: selected_seed")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let img_size = IMG_SIZE;
    let grid_size = img_size * 1.6;
    let father = None;
    let father_cell = None;
    let father_elm = document
        .get_element_by_id("father")
        .expect("Error!: father")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let mother = None;
    let mother_cell = None;
    let mother_elm = document
        .get_element_by_id("mother")
        .expect("Error!: father")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let inp_mode_menu = document
        .get_element_by_id("mode-menu")
        .expect("Error!: inp_mode_menu")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let inp_mode_seed = document
        .get_element_by_id("mode-seed")
        .expect("Error!: inp_mode_seed")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let inp_mode_hybrid = document
        .get_element_by_id("mode-hybrid")
        .expect("Error!: inp_mode_hybrid")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let inp_mode_remove = document
        .get_element_by_id("mode-remove")
        .expect("Error!: inp_mode_remove")
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    let mousedown_startpos = (0, 0);
    let grabed_img = None;
    let pallet = document
        .get_element_by_id("pallet")
        .expect("Error!: pallet")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let pallet_inp_color = document
        .get_element_by_id("pallet-filter")
        .expect("pallet filter")
        .dyn_into::<HtmlSelectElement>()
        .unwrap();

    let overlay = document
        .get_element_by_id("overlay")
        .expect("overlay")
        .dyn_into::<HtmlElement>()
        .unwrap();
    /* let modal = document
    .get_element_by_id("modal")
    .expect("overlay")
    .dyn_into::<HtmlElement>()
    .unwrap(); */

    let singleton = Singleton {
        window,
        document,
        editor,
        contextmenu,
        // contextmenu_is_popuped,
        contextmenu_pos,
        contextmenu_candidate_flowers,
        hybrid_from_index,
        canvas,
        /*
        cell_book,
        hybrid_book,
         */
        canvas_book,
        inp_filename,
        inp_kind,
        inp_genome,
        selected_seed_cell,
        selected_seed_elm,
        img_size,
        grid_size,
        father,
        father_cell,
        father_elm,
        mother,
        mother_cell,
        mother_elm,
        inp_mode_menu,
        inp_mode_seed,
        inp_mode_hybrid,
        inp_mode_remove,
        /*
        father_img,
        father_str,
        mother_img,
        mother_str,
        */
        mousedown_startpos,
        grabed_img,
        pallet,
        pallet_inp_color,
        overlay,
        // modal,
    };
    let singleton = Rc::new(RefCell::new(singleton));
    let sin = singleton.clone();
    // console_log!("[779] borrow_mut");
    let mut s = sin.borrow_mut();
    // console_log!("[781] borrow_mut");

    load_hash(&mut s)?;
    init_filename(&s, &singleton)?;
    init_window(&s, &singleton)?;
    init_seed_form(&mut s, &singleton)?;
    init_editor(&s, &singleton)?;
    init_mode_wrappers(&s, &singleton)?;
    init_seed_mode(&s, &singleton)?;
    cursor_set(&s)?;
    init_radio_inputs(&s, &singleton)?;
    init_pallet(&s, &singleton)?;
    init_overlay(&s, &singleton)?;
    init_all_delete(&s, &singleton)?;
    init_save_mode(&s, &singleton)?;

    Ok(())
}

fn load_hash(s: &mut Singleton) -> Result<(), JsValue> {
    let hash = s.window.location().hash()?;
    if hash.len() > 0 {
        let code = hash.replace("#", "");
        let mut hash_book = None;
        if let Ok(canvas_book) = CanvasBook::load(s, code.as_str()) {
            hash_book = Some(canvas_book);
        }
        if let Some(canvas_book) = hash_book {
            // plant_flower_from_cell_book(&cell_book, &s)?;
            s.canvas_book = canvas_book;
            s.canvas_book.plant_flower(s)?;
        }
    }

    Ok(())
}

fn init_filename(s: &Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let save_filename = s
        .document
        .get_element_by_id("save-filename")
        .expect("Error!: save-filename")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        save_filename.set_value(&s.inp_filename.value());
        s.canvas_book.save()?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    s.inp_filename.set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    Ok(())
}

fn init_window(s: &Singleton, singleton: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let f_s_ref = singleton.clone();
    let f = Closure::wrap(Box::new(move |_: MouseEvent| -> Result<(), JsValue> {
        // console_log!("[785] borrow_mut");
        let mut s = f_s_ref.borrow_mut();
        // console_log!("[787] borrow_mut");
        s.grabed_img = None;

        s.contextmenu_pos = None;
        s.contextmenu.set_css_styles(vec![("display", "none")])?;
        // console_log!("document {:?}", e.target());
        s.pallet.style().set_property("display", "none")?;
        Ok(())
    }) as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);
    s.document
        .add_event_listener_with_callback("click", f.as_ref().unchecked_ref())?;

    f.forget();

    let f_s_ref = singleton.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let mut s = f_s_ref.borrow_mut();
        s.grabed_img = None;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    s.document
        .add_event_listener_with_callback("mouseup", f.as_ref().unchecked_ref())?;
    f.forget();

    let f_s_ref = singleton.clone();
    let set_canvas = move || -> Result<(), JsValue> {
        // console_log!("[798] borrow_mut");
        let s = f_s_ref.borrow();
        // console_log!("[800] borrow_mut");
        draw_canvas(&s)?;
        Ok(())
    };

    // set_canvas()?;
    draw_canvas(&s)?;

    let f = Closure::wrap(Box::new(set_canvas) as Box<dyn FnMut() -> Result<(), JsValue>>);
    s.window.set_onresize(Some(f.as_ref().unchecked_ref()));
    f.forget(); // cation: memory leak

    let f_s = singleton.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let mut s = f_s.borrow_mut();
        load_hash(&mut s)?;
        draw_canvas(&s)?;

        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    s.window.set_onhashchange(Some(f.as_ref().unchecked_ref()));
    f.forget();

    let f_s = singleton.clone();
    let f = Closure::wrap(Box::new(move |e: KeyboardEvent| -> Result<(), JsValue> {
        let s = f_s.borrow();
        if e.ctrl_key() {
            match e.key().as_str() {
                "z" => s.window.history()?.back()?,
                "y" => s.window.history()?.forward()?,
                _ => (),
            }
        }
        Ok(())
    })
        as Box<dyn FnMut(KeyboardEvent) -> Result<(), JsValue>>);
    s.window.set_onkeydown(Some(f.as_ref().unchecked_ref()));
    f.forget();

    Ok(())
}

fn init_editor(s: &Singleton, singleton: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let f_s_rc = singleton.clone();
    let f = Closure::wrap(Box::new(move |e: MouseEvent| -> Result<(), JsValue> {
        let mut s = f_s_rc.borrow_mut();

        let dom_rect = s.editor.get_bounding_client_rect();
        let mx = e.client_x() as f64 - dom_rect.x();
        let my = e.client_y() as f64 - dom_rect.y();
        let x = (mx / s.grid_size) as usize;
        let y = (my / s.grid_size) as usize;

        s.mousedown_startpos = (x, y);

        if let Some(CanvasCell::FlowerCell { flower, elm: _ }) = s.canvas_book.c_get(&(x, y)) {
            let elm = s
                .document
                .create_element("img")?
                .dyn_into::<HtmlImageElement>()
                .unwrap();
            elm.set_src(format!("{}.png", flower.get_name()).as_str());
            elm.set_class_name("grabed-img");
            elm.set_css_styles(vec![
                ("position", "absolute"),
                ("left", format!("{}px", mx).as_str()),
                ("top", format!("{}px", my).as_str()),
            ])?;
            s.editor.append_child(&elm)?;
            s.grabed_img = Some(GrabedImg(elm));
        }

        Ok(())
    }) as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);

    s.editor
        .add_event_listener_with_callback("mousedown", f.as_ref().unchecked_ref())?;

    f.forget();

    let f_s_rc = singleton.clone();
    let f = Closure::wrap(Box::new(move |e: MouseEvent| -> Result<(), JsValue> {
        let s = f_s_rc.borrow_mut();
        match &s.grabed_img {
            Some(gi) => {
                let dom_rect = s.editor.get_bounding_client_rect();
                let mx = e.client_x() as f64 - dom_rect.x();
                let my = e.client_y() as f64 - dom_rect.y();

                gi.0.set_css_styles(vec![
                    ("left", format!("{}px", mx).as_str()),
                    ("top", format!("{}px", my).as_str()),
                ])?;
            }
            None => (),
        }
        Ok(())
    }) as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);

    s.editor
        .add_event_listener_with_callback("mousemove", f.as_ref().unchecked_ref())?;

    f.forget();

    let f_s_rc = singleton.clone();
    let f = Closure::wrap(Box::new(move |e: MouseEvent| -> Result<(), JsValue> {
        let s_rc = f_s_rc.clone();
        // console_log!("[816] borrow_mut");
        let mut s = f_s_rc.borrow_mut();
        // console_log!("[818] borrow_mut");

        let dom_rect = s.editor.get_bounding_client_rect();
        let x = e.client_x() as f64 - dom_rect.x();
        let y = e.client_y() as f64 - dom_rect.y();
        let x = (x / s.grid_size) as usize;
        let y = (y / s.grid_size) as usize;

        if (x, y) != s.mousedown_startpos {
            let p1 = s.mousedown_startpos;
            let p2 = (x, y);
            if let Some(CanvasCell::FlowerCell { flower: _, elm: _ }) = s.canvas_book.c_get(&p1) {
                if s.canvas_book.c_get(&p2).is_none() {
                    let cell = s.canvas_book.c_remove(&p1).unwrap();
                    plant_flower(p2.0, p2.1, &mut s, cell.get_flower().unwrap().clone())?;
                } else if s.canvas_book.c_get(&p2).unwrap().is_flower_cell() {
                    let dx = cmp::max(p1.0, p2.0) - cmp::min(p1.0, p2.0);
                    let dy = cmp::max(p1.1, p2.1) - cmp::min(p1.1, p2.1);
                    let p3 = if dx <= dy {
                        let x = cmp::max(p1.0, p2.0) + 2;
                        let y = (p1.1 + p2.1) / 2;
                        (x, y)
                    } else {
                        let x = (p1.0 + p2.0) / 2;
                        let y = cmp::max(p1.1, p2.1) + 2;
                        (x, y)
                    };
                    let ht = HT(p1, p2, p3);
                    let hl_w = HybridLine::new(ht, &mut s.canvas_book.cell_book);
                    if_chain! {
                        if let Ok(hl) = hl_w;
                        if s.canvas_book.h_get(&ht).is_none();
                        then {
                            s.canvas_book.h_insert(ht, hl);
                        }
                    }
                }
            }
            // draw_canvas(&s)?;
            return Ok(());
        }

        if s.inp_mode_menu.checked() {
            // f_hfi.set(Some(0));
            let flag = if let Some(pos) = s.contextmenu_pos {
                pos != (x, y)
            } else {
                true
            };

            if flag {
                contextmenu_popup(x, y, &mut s, &s_rc)?;
                e.stop_propagation();
            }
        }

        // if s.contextmenu_is_popuped {
        if s.contextmenu_pos.is_some() {
            return Ok(());
        }

        if s.inp_mode_seed.checked() {
            let flower_w =
                Flower::strs2flower(s.inp_kind.value().as_str(), s.inp_genome.value().as_str());
            if flower_w.is_none() {
                return Ok(());
            }
            let flower = flower_w.unwrap();
            plant_flower(x, y, &mut s, flower)?;
        } else if s.inp_mode_hybrid.checked() {
            parents_event(x, y, &mut s)?;
        } else if s.inp_mode_remove.checked() {
            remove_cell(x, y, &mut s)?;
        }

        // draw_canvas(&s)?;

        Ok(())
    }) as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);

    s.editor
        .add_event_listener_with_callback("click", f.as_ref().unchecked_ref())?;

    f.forget(); // cation: memory leak

    let f_s_rc = singleton.clone();
    let f = Closure::wrap(Box::new(move |e: MouseEvent| -> Result<(), JsValue> {
        let s_rc = f_s_rc.clone();
        // console_log!("[857] borrow_mut");
        let mut s = f_s_rc.borrow_mut();
        // console_log!("[859] borrow_mut");

        let dom_rect = s.editor.get_bounding_client_rect();
        let x = e.client_x() as f64 - dom_rect.x();
        let y = e.client_y() as f64 - dom_rect.y();
        let x = (x / s.grid_size) as usize;
        let y = (y / s.grid_size) as usize;
        contextmenu_popup(x, y, &mut s, &s_rc)?;
        Ok(())
    }) as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);
    s.editor
        .add_event_listener_with_callback("contextmenu", f.as_ref().unchecked_ref())?;
    f.forget();

    Ok(())
}

fn init_mode_wrappers(s: &Singleton, singleton: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let targets = js_sys::Array::from(
        s.document
            .get_elements_by_class_name("mode-wrapper")
            .dyn_into::<JsValue>()?
            .as_ref(),
    )
    .to_vec()
    .into_iter()
    .map(JsValue::dyn_into::<HtmlElement>)
    .collect::<Result<Vec<HtmlElement>, JsValue>>()?;

    for (i, target) in targets.into_iter().enumerate() {
        let mode_input_w = target.query_selector("input[name='mode']")?;
        if mode_input_w.is_none() {
            continue;
        }
        let mode_input = mode_input_w
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let a_s_rc = singleton.clone();
        let a = Closure::wrap(Box::new(move |e: MouseEvent| -> Result<(), JsValue> {
            let mut s = a_s_rc.borrow_mut();
            if mode_input.id() != "mode-hybrid" {
                // console_log!("[897] borrow_mut");
                reset_parents(&mut s)?;
                // console_log!("[899] borrow_mut");
            }
            // mode_input.set_checked(true);
            // ^ ラベルで対処する
            s.grabed_img = None;
            s.contextmenu_pos = None;
            s.contextmenu.set_css_styles(vec![("display", "none")])?;
            // console_log!("mode input {:?}", e.target());
            if i != 1 {
                s.pallet.set_css_styles(vec![("display", "none")])?;
            }
            e.stop_propagation();
            Ok(())
        })
            as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);
        target.add_event_listener_with_callback("click", a.as_ref().unchecked_ref())?;
        a.forget();
    }

    Ok(())
}

fn init_seed_mode(s: &Singleton, singleton: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let flower_kind_elm = s
        .document
        .get_element_by_id("contextmenu-flower-kind")
        .expect("Error!: contextmenu-flower-kind")
        .dyn_into::<HtmlSelectElement>()
        .unwrap();

    let f = Closure::wrap(Box::new(move |e: Event| -> Result<(), JsValue> {
        e.stop_propagation();
        Ok(())
    }) as Box<dyn FnMut(Event) -> Result<(), JsValue>>);
    flower_kind_elm.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    for (i, color) in vec!["red", "yellow", "white"].into_iter().enumerate() {
        let f_s = singleton.clone();
        let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
            let mut s = f_s.borrow_mut();
            let kind =
                FlowerKind::from_str(s.inp_kind.value().as_str()).unwrap_or(FlowerKind::Rose);
            let seed_flowers = Flower::get_seed_flowers(kind);
            update_seed_form(seed_flowers.to_vec()[i], &mut s)?;

            Ok(())
        }) as Box<dyn FnMut() -> Result<(), JsValue>>);
        let seed = s
            .document
            .get_element_by_id(format!("{}-seed", color).as_str())
            .expect(format!("Error!: {}-seed", color).as_str())
            .dyn_into::<HtmlElement>()
            .unwrap();
        seed.set_onclick(Some(f.as_ref().unchecked_ref()));
        f.forget();
    }

    Ok(())
}

fn init_radio_inputs(s: &Singleton, singleton: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let inp_list = s.document.query_selector_all("input[name='mode']")?;

    for i in 0..inp_list.length() {
        let inp_mode = inp_list
            .get(i as u32)
            .expect("Error!: no mode node")
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let f_s = singleton.clone();
        let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
            let s = f_s.borrow();
            cursor_set(&s)?;
            Ok(())
        }) as Box<dyn FnMut() -> Result<(), JsValue>>);
        inp_mode.set_oninput(Some(f.as_ref().unchecked_ref()));
        f.forget();
    }

    Ok(())
}

fn draw_canvas(s: &Singleton) -> Result<(), JsValue> {
    let w = s
        .window
        .get_computed_style(&s.editor)?
        .unwrap()
        .get_property_value("width")?;
    // // console_log!("width: '{}'", w);
    // canvas.set_width();
    let w = w.replace("px", "").parse::<f64>().unwrap() as u32;
    s.canvas.set_width(w);

    let h = s
        .window
        .get_computed_style(&s.editor)?
        .unwrap()
        .get_property_value("height")?;
    // // console_log!("height: '{}'", h);
    // canvas.set_width();
    let h = h.replace("px", "").parse::<f64>().unwrap() as u32;
    s.canvas.set_height(h);

    let img_size = IMG_SIZE;
    let grid_size = img_size * 1.6;

    canvas_set_grid(&s.canvas, grid_size)?;

    let ctx = s
        .canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    // V for Debug

    /*
    for (&key, val) in s.cell_book.iter() {
        let color = match val {
            CanvasCell::FlowerCell { flower: _, elm: _ } => "#FF0000",
            CanvasCell::HybridLineCell { hybrids: _ } => "#008000",
        };

        let (x, y) = (
            (key.0 as f64 + 0.25) * grid_size,
            (key.1 as f64 + 0.25) * grid_size,
        );
        let (w, h) = (grid_size / 2_f64, grid_size / 2_f64);

        ContextChain::new(&ctx)
            .set_fill_style(&JsValue::from(color))
            .begin_path()
            .fill_rect(x, y, w, h)
            .stroke();
    }

     */
    // ^ for Debug

    // ctx.set_stroke_style(&JsValue::from("#0000FF"));
    // ctx.set_line_width(3_f64);

    for (_key, val) in s.canvas_book.h_iter() {
        (val.draw_line)(&ctx, grid_size);
    }

    for (_key, val) in s.canvas_book.h_iter() {
        (val.draw_chance)(&ctx, grid_size, &s.canvas_book.cell_book);
    }

    // save(&s)?;

    Ok(())
}

fn canvas_set_grid(canvas: &HtmlCanvasElement, interval: f64) -> Result<(), JsValue> {
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    let mut ctx_chain = ContextChain::new(&context)
        .begin_path()
        .set_stroke_style(&JsValue::from("#eeeeee"))
        .set_line_width(1_f64);

    /*
    context.set_stroke_style(&JsValue::from("#eeeeee"));
    context.set_line_width(1_f64);
     */

    let mut p = 0_f64;
    while p < canvas.width() as f64 {
        ctx_chain = ctx_chain
            .move_to(p, 0_f64)
            .line_to(p, canvas.height() as f64);
        p += interval;
    }

    let mut p = 0_f64;
    while p < canvas.height() as f64 {
        ctx_chain = ctx_chain
            .move_to(0_f64, p)
            .line_to(canvas.width() as f64, p);
        p += interval;
    }

    ctx_chain.stroke();
    Ok(())
}

fn init_seed_form(s: &mut Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let flower_w = Flower::strs2flower(s.inp_kind.value().as_str(), s.inp_genome.value().as_str());
    if let Some(flower) = flower_w {
        /*
        selected_seed_img.set_src(format!("{}.png", flower.get_name()).as_str());
        selected_seed_str.set_inner_html(flower.to_string().as_str());
         */
        let flower_cell = CanvasCell::new_flower(flower, &s.document, s.img_size)?;
        s.selected_seed_elm
            .append_child(&flower_cell.get_elm().unwrap())?;
        s.selected_seed_cell = Some(flower_cell);
    }

    let mut funcs = (0..2)
        .map(|_| {
            let a_s = s_rc.clone();
            /*
            let a_seed_img = selected_seed_img.clone();
            let a_seed_str = selected_seed_str.clone();
             */
            let a = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
                // console_log!("[1064] borrow_mut");
                let mut s = a_s.borrow_mut();
                // console_log!("[1066] borrow_mut");

                let flower_w =
                    Flower::strs2flower(s.inp_kind.value().as_str(), s.inp_genome.value().as_str());
                if let Some(flower) = flower_w {
                    /*
                    a_seed_img.set_src(format!("{}.png", flower.get_name()).as_str());
                    a_seed_str.set_inner_html(flower.to_string().as_str());
                     */
                    let flower_cell = CanvasCell::new_flower(flower, &s.document, s.img_size)?;
                    s.selected_seed_elm
                        .append_child(&flower_cell.get_elm().unwrap())?;
                    s.selected_seed_cell = Some(flower_cell);
                }
                set_pallet_contents(&s)?;

                Ok(())
            }) as Box<dyn FnMut() -> Result<(), JsValue>>);
            a
        })
        .collect::<Vec<_>>();

    let (a, b) = (funcs.pop().unwrap(), funcs.pop().unwrap());
    s.inp_kind.set_oninput(Some(a.as_ref().unchecked_ref()));
    a.forget();
    s.inp_genome.set_oninput(Some(b.as_ref().unchecked_ref()));
    b.forget();

    Ok(())
}

fn update_seed_form(flower: Flower, s: &mut Singleton) -> Result<(), JsValue> {
    s.inp_kind.set_value(flower.kind.to_string().as_str());
    s.inp_genome.set_value(flower.to_string().as_str());
    let flower_cell = CanvasCell::new_flower(flower, &s.document, s.img_size)?;
    s.selected_seed_elm
        .append_child(&flower_cell.get_elm().unwrap())?;
    s.selected_seed_cell = Some(flower_cell);
    Ok(())
}

fn render_flower(x: usize, y: usize, elm: &HtmlElement, s: &Singleton) -> Result<(), JsValue> {
    elm.class_list().add(&js_sys::Array::from_iter(
        vec![&JsValue::from("have-border")].into_iter(),
    ))?;
    elm.set_css_styles(vec![
        ("position", "absolute"),
        (
            "top",
            format!("{}px", s.grid_size * y as f64 - 3_f64).as_str(),
        ),
        (
            "left",
            format!("{}px", s.grid_size * x as f64 - 3_f64).as_str(),
        ),
    ])?;

    s.editor.append_child(elm)?;
    Ok(())
}

fn plant_flower(x: usize, y: usize, s: &mut Singleton, flower: Flower) -> Result<(), JsValue> {
    if let Some(CanvasCell::HybridLineCell { hybrids: _ }) = s.canvas_book.c_get(&(x, y)) {
        return Ok(());
    }

    update_seed_form(flower, s)?;

    let flower_cell = CanvasCell::new_flower(flower, &s.document, s.img_size)?;
    let elm = flower_cell.get_elm().unwrap();
    render_flower(x, y, elm, s)?;
    s.canvas_book.c_insert((x, y), flower_cell);

    Ok(())
}

fn move_to_other_mode(s: &Singleton, i: usize) -> Result<(), JsValue> {
    let inp_mode = s
        .document
        .query_selector_all("input[name='mode']")?
        .get(i as u32)
        .expect("Error!: no mode node")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    inp_mode.set_checked(true);

    cursor_set(s)?;

    Ok(())
}

fn parents_event(x: usize, y: usize, s: &mut Singleton) -> Result<(), JsValue> {
    cursor_set(s)?;
    let canvas_cell = match s.canvas_book.c_get(&(x, y)) {
        Some(c) => c,
        None => {
            if_chain! {
                if let Some(father_pos) = s.father;
                if let Some(mother_pos) = s.mother;
                then {
                    // // console_log!("{:?}", Dir::decide_dir(father_pos, mother_pos, (x, y)));
                    let ht = HT(father_pos, mother_pos, (x, y));
                    let hl_w = HybridLine::new(ht, &mut s.canvas_book.cell_book);
                    if_chain! {
                        if let Ok(hl) = hl_w;
                        if s.canvas_book.h_get(&ht).is_none();
                        then {
                            s.canvas_book.h_insert(ht, hl);
                        }
                    }
                }
            }
            if let Some(i) = s.hybrid_from_index {
                move_to_other_mode(s, i)?;
            }
            reset_parents(s)?;
            return Ok(());
        }
    };
    let (flower, elm) = match canvas_cell {
        CanvasCell::FlowerCell { flower, elm } => (flower, elm),
        CanvasCell::HybridLineCell { hybrids: _ } => {
            if let Some(i) = s.hybrid_from_index {
                move_to_other_mode(s, i)?;
            }
            reset_parents(s)?;
            return Ok(());
        }
    };

    let father_pos = match s.father {
        Some(p) => p,
        None => {
            s.father = Some((x, y));
            elm.class_list().add(&js_sys::Array::from_iter(
                vec![&JsValue::from("watered")].into_iter(),
            ))?;
            /*
            s.father_str.set_inner_html(flower.to_string().as_str());
            s.father_img
                .set_src(format!("{}.png", flower.get_name()).as_str());
             */
            let flower_cell = CanvasCell::new_flower(flower.clone(), &s.document, s.img_size)?;
            s.father_elm.append_child(&flower_cell.get_elm().unwrap())?;
            s.father_cell = Some(flower_cell);
            return Ok(());
        }
    };

    let mother_pos = match s.mother {
        Some(p) => p,
        None => {
            s.mother = Some((x, y));
            elm.class_list().add(&js_sys::Array::from_iter(
                vec![&JsValue::from("watered")].into_iter(),
            ))?;
            /*
            s.mother_str.set_inner_html(flower.to_string().as_str());
            s.mother_img
                .set_src(format!("{}.png", flower.get_name()).as_str());
             */
            let flower_cell = CanvasCell::new_flower(flower.clone(), &s.document, s.img_size)?;
            s.mother_elm.append_child(&flower_cell.get_elm().unwrap())?;
            s.mother_cell = Some(flower_cell);
            return Ok(());
        }
    };
    // console_log!("{:?}", Dir::decide_dir(father_pos, mother_pos, (x, y)));
    let ht = HT(father_pos, mother_pos, (x, y));
    let hl_w = HybridLine::new(ht, &mut s.canvas_book.cell_book);
    if_chain! {
        if let Ok(hl) = hl_w;
        if s.canvas_book.h_get(&ht).is_none();
        then {
            s.canvas_book.h_insert(ht, hl);
        }
    }
    if let Some(i) = s.hybrid_from_index {
        move_to_other_mode(s, i)?;
    }
    reset_parents(s)?;

    Ok(())
}

fn reset_parents(s: &mut Singleton) -> Result<(), JsValue> {
    js_sys::Array::from(&s.document.get_elements_by_class_name("watered"))
        .to_vec()
        .into_iter()
        .map(JsValue::dyn_into::<HtmlElement>)
        .filter(Result::is_ok)
        .for_each(|e| {
            let _ = e.unwrap().class_list().remove(&js_sys::Array::from_iter(
                vec![&JsValue::from("watered")].into_iter(),
            ));
        });
    s.father = None;
    s.mother = None;
    s.father_cell = None;
    s.mother_cell = None;
    /*
    s.father_img.set_src("");
    s.father_str.set_inner_html("");
    s.mother_img.set_src("");
    s.mother_str.set_inner_html("");
     */
    s.hybrid_from_index = None;

    Ok(())
}

fn remove_cell(x: usize, y: usize, s: &mut Singleton) -> Result<(), JsValue> {
    let target_hybrids;
    match s.canvas_book.c_get(&(x, y)) {
        Some(CanvasCell::FlowerCell { flower: _, elm: _ }) => {
            s.canvas_book.c_remove(&(x, y));
            return Ok(());
        }
        Some(CanvasCell::HybridLineCell { hybrids }) => {
            // hybrids.iter().map(|&ht| ht).collect::<Vec<_>>()
            target_hybrids = hybrids.clone();
        }
        _ => return Ok(()),
    }

    target_hybrids.iter().for_each(|&ht| {
        let mut pos_array = Vec::new();
        if let Some(hl) = s.canvas_book.hybrid_book.get(&ht) {
            pos_array = hl.occupied_pos.clone();
        }
        for p in pos_array.into_iter() {
            let is_empty = if let Some(cell) = s.canvas_book.c_get_mut(&p) {
                cell.remove_hybrid(&ht)
            } else {
                false
            };

            if is_empty {
                s.canvas_book.c_remove(&p);
            }
        }
    });

    for ht in target_hybrids.into_iter() {
        s.canvas_book.h_remove(&ht);
    }

    Ok(())
}

fn contextmenu_popup(
    x: usize,
    y: usize,
    s: &mut Singleton,
    s_rc: &Rc<RefCell<Singleton>>,
) -> Result<(), JsValue> {
    // s.contextmenu_is_popuped = true;
    s.contextmenu_pos = Some((x, y));
    s.contextmenu.set_css_styles(vec![
        ("display", "block"),
        ("left", format!("{}px", (x as f64) * s.grid_size).as_str()),
        (
            "top",
            format!("{}px", (y as f64 + 1_f64) * s.grid_size).as_str(),
        ),
        /*
        (
            "width",
            format!("{}px", s.grid_size * 3_f64 + 20_f64).as_str(),
        ),
         */
    ])?;

    let delete = s
        .contextmenu
        .query_selector("#contextmenu-delete")?
        .expect("Error!: contextmenu-delete")
        .dyn_into::<HtmlElement>()
        .unwrap();

    let should_show_cd = s.canvas_book.c_get(&(x, y)).is_some();

    delete
        .style()
        .set_property("display", if should_show_cd { "block" } else { "none" })?;

    let f_s_rc = s_rc.clone();
    let f = if should_show_cd {
        Box::new(move |e: Event| -> Result<(), JsValue> {
            // console_log!("[1325] borrow_mut");
            let mut s = f_s_rc.borrow_mut();
            // console_log!("[1327] borrow_mut");
            remove_cell(x, y, &mut s)?;

            // s.contextmenu_is_popuped = false;
            s.contextmenu_pos = None;
            s.contextmenu.set_css_styles(vec![("display", "none")])?;
            e.stop_propagation();

            // draw_canvas(&s)?;
            Ok(())
        }) as Box<dyn FnMut(Event) -> Result<(), JsValue>>
    } else {
        Box::new(move |_: Event| -> Result<(), JsValue> { Ok(()) })
            as Box<dyn FnMut(Event) -> Result<(), JsValue>>
    };
    let f = Closure::wrap(f);
    // let mut option = AddEventListenerOptions::new();
    /*
    delete.add_event_listener_with_callback_and_add_event_listener_options(
        "click",
        f.as_ref().unchecked_ref(),
        option.once(true),
    )?;
     */
    delete.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    let hybrid_elm = s
        .contextmenu
        .query_selector("#contextmenu-hybrid")?
        .expect("Error!: contextmenu-hybrid")
        .dyn_into::<HtmlElement>()
        .unwrap();

    let should_show_ch = s
        .canvas_book
        .c_get(&(x, y))
        .and_then(|cell| {
            if cell.is_flower_cell() {
                Some(())
            } else {
                None
            }
        })
        .is_some();

    hybrid_elm
        .style()
        .set_property("display", if should_show_ch { "block" } else { "none" })?;

    let f_s_rc = s_rc.clone();
    let f = if should_show_ch {
        Box::new(move |e: Event| -> Result<(), JsValue> {
            // console_log!("[1351] borrow_mut");
            let mut s = f_s_rc.borrow_mut();
            // console_log!("[1353] borrow_mut");
            s.hybrid_from_index = if s.inp_mode_menu.checked() {
                Some(0)
            } else if s.inp_mode_seed.checked() {
                Some(1)
            } else if s.inp_mode_hybrid.checked() {
                Some(2)
            } else if s.inp_mode_remove.checked() {
                Some(3)
            } else {
                None
            };
            s.inp_mode_hybrid.set_checked(true);
            parents_event(x, y, &mut s)?;

            // s.contextmenu_is_popuped = false;
            s.contextmenu_pos = None;
            s.contextmenu.set_css_styles(vec![("display", "none")])?;
            e.stop_propagation();

            // draw_canvas(&s)?;
            Ok(())
        }) as Box<dyn FnMut(Event) -> Result<(), JsValue>>
    } else {
        Box::new(move |_: Event| -> Result<(), JsValue> { Ok(()) })
            as Box<dyn FnMut(Event) -> Result<(), JsValue>>
    };
    let f = Closure::wrap(f);

    hybrid_elm.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    let flower_kind_elm = Rc::new(
        s.document
            .get_element_by_id("contextmenu-flower-kind")
            .expect("Error!: contextmenu-flower-kind")
            .dyn_into::<HtmlSelectElement>()
            .unwrap(),
    );
    let container_elm = Rc::new(
        s.contextmenu
            .query_selector("#contextmenu-flowers-container")?
            .expect("Error!: contextmenu-flowers-container")
            .dyn_into::<HtmlElement>()
            .unwrap(),
    );

    container_elm.style().set_property("display", "flex")?;
    if_chain! {
        if let Some(c) = s.canvas_book.c_get(&(x, y));
        if c.is_hybrid_cell();
        then {
            flower_kind_elm.style().set_property("display", "none")?;
            container_elm.style().set_property("display", "none")?;
            return Ok(());
        }
    }

    let p = (x, y);
    let mut cands = HashSet::new();
    for (&ht, _) in s.canvas_book.h_iter() {
        let HT(fp, mp, cp) = ht;

        if fp == p {
            if_chain! {
                if let Some(m) = s.canvas_book.c_get(&mp);
                if let Some(mf) = m.get_flower();
                if let Some(c) = s.canvas_book.c_get(&cp);
                if let Some(cf) = c.get_flower();
                if let Some(v) = cf.get_other_parents(mf);
                then {
                    for (_, flower) in v.into_iter() {
                        cands.insert(flower);
                    }
                }
            }
        } else if mp == p {
            if_chain! {
                if let Some(f) = s.canvas_book.c_get(&fp);
                if let Some(ff) = f.get_flower();
                if let Some(c) = s.canvas_book.c_get(&cp);
                if let Some(cf) = c.get_flower();
                if let Some(v) = cf.get_other_parents(ff);
                then {
                    for (_, flower) in v.into_iter() {
                        cands.insert(flower);
                    }
                }
            }
        } else if cp == p {
            if_chain! {
                if let Some(m) = s.canvas_book.c_get(&mp);
                if let Some(mf) = m.get_flower();
                if let Some(f) = s.canvas_book.c_get(&fp);
                if let Some(ff) = f.get_flower();
                if let Some(v) = ff.hybrid_with(mf);
                then {
                    for (_, flower) in v.into_iter() {
                        cands.insert(flower);
                    }
                }
            }
        }
    }

    let require_filter = cands.is_empty();
    if require_filter {
        for k in FlowerKind::vec().into_iter() {
            let SeedFlowers { red, yellow, white } = Flower::get_seed_flowers(k);
            for flower in vec![red, yellow, white].into_iter() {
                cands.insert(flower);
            }
        }

        s.canvas_book
            .c_iter()
            .filter(|(_, cell)| cell.is_flower_cell())
            .for_each(|(_, cell)| {
                let flower = cell.get_flower().unwrap().clone();
                cands.insert(flower);
            });
    }

    flower_kind_elm
        .style()
        .set_property("display", if require_filter { "inline" } else { "none" })?;
    let default_kind_str = s.inp_kind.value();
    flower_kind_elm.set_value(default_kind_str.as_str());
    let default_kind = FlowerKind::from_str(default_kind_str.as_str()).unwrap_or(FlowerKind::Rose);

    container_elm.set_inner_html("");

    let mut cands = cands.into_iter().collect::<Vec<_>>();
    let mut counter = 0;
    cands.sort();
    cands.reverse();
    let cands = cands
        .into_iter()
        .map(|flower| -> Result<CanvasCell, JsValue> {
            let fc = CanvasCell::new_flower(flower, &s.document, s.img_size)?;
            let elm = fc.get_elm().unwrap();
            elm.class_list().add(&js_sys::Array::from_iter(
                vec![
                    &JsValue::from("candidate-flower"),
                    &JsValue::from("clickable"),
                ]
                .into_iter(),
            ))?;
            // elm.set_css_styles(vec![("border-style", "none")])?;
            if require_filter && flower.kind != default_kind {
                elm.set_css_styles(vec![("display", "none")])?;
            } else if require_filter {
                counter += 1;
            }
            let f_s = s_rc.clone();
            let f = Closure::wrap(Box::new(move |e: Event| -> Result<(), JsValue> {
                let mut s = f_s.borrow_mut();
                plant_flower(x, y, &mut s, flower)?;

                // s.contextmenu_is_popuped = false;
                s.contextmenu_pos = None;
                s.contextmenu.set_css_styles(vec![("display", "none")])?;
                e.stop_propagation();

                draw_canvas(&s)?;
                Ok(())
            })
                as Box<dyn FnMut(Event) -> Result<(), JsValue>>);
            elm.set_onclick(Some(f.as_ref().unchecked_ref()));
            f.forget();
            container_elm.append_child(elm)?;
            Ok(fc)
        })
        .collect::<Result<Vec<_>, JsValue>>()?;

    /*
    let width = cands[0].get_elm().unwrap().client_width();
    s.contextmenu
        .style()
        .set_property("width", format!("{}px", width * 3).as_str())?;
     */

    /*
    if cands.len() > 6 {
        s.contextmenu.style().set_property(
            "width",
            format!("{}px", s.grid_size * 3_f64 + 15_f64).as_str(),
        )?;
    }
     */

    let t = if require_filter { counter } else { cands.len() };

    if t <= 3 {
        s.contextmenu
            .style()
            .set_property("width", format!("{}px", s.grid_size * 3_f64).as_str())?;
        container_elm.set_css_styles(vec![
            ("height", format!("{}px", s.grid_size).as_str()),
            ("overflow", "visible"),
        ])?;
    } else if t <= 6 {
        s.contextmenu
            .style()
            .set_property("width", format!("{}px", s.grid_size * 3_f64).as_str())?;
        container_elm.set_css_styles(vec![
            ("height", format!("{}px", s.grid_size * 2_f64).as_str()),
            ("overflow", "visible"),
        ])?;
    } else {
        s.contextmenu.style().set_property(
            "width",
            format!("{}px", s.grid_size * 3_f64 + 20_f64).as_str(),
        )?;
        container_elm.set_css_styles(vec![
            ("height", format!("{}px", s.grid_size * 2_f64).as_str()),
            ("overflow", "scroll"),
        ])?;
    }

    s.contextmenu_candidate_flowers = cands;

    let f_kind_elm = flower_kind_elm.clone();
    let f_container_elm = container_elm.clone();
    let f_s = s_rc.clone();
    let f = if require_filter {
        Box::new(move |_: Event| -> Result<(), JsValue> {
            let mut s = f_s.borrow_mut();
            let kind =
                FlowerKind::from_str(f_kind_elm.value().as_str()).unwrap_or(FlowerKind::Rose);
            let cands = &mut s.contextmenu_candidate_flowers;
            let mut counter = 0;
            for fc in cands.iter() {
                let flower = fc.get_flower().unwrap();
                let elm = fc.get_elm().unwrap();
                let val = if flower.kind == kind {
                    counter += 1;
                    "inline"
                } else {
                    "none"
                };
                elm.style().set_property("display", val)?;
            }

            if counter <= 3 {
                s.contextmenu
                    .style()
                    .set_property("width", format!("{}px", s.grid_size * 3_f64).as_str())?;
                f_container_elm.set_css_styles(vec![
                    ("height", format!("{}px", s.grid_size).as_str()),
                    ("overflow", "visible"),
                ])?;
            } else if counter <= 6 {
                s.contextmenu
                    .style()
                    .set_property("width", format!("{}px", s.grid_size * 3_f64).as_str())?;
                f_container_elm.set_css_styles(vec![
                    ("height", format!("{}px", s.grid_size * 2_f64).as_str()),
                    ("overflow", "visible"),
                ])?;
            } else {
                s.contextmenu.style().set_property(
                    "width",
                    format!("{}px", s.grid_size * 3_f64 + 20_f64).as_str(),
                )?;
                f_container_elm.set_css_styles(vec![
                    ("height", format!("{}px", s.grid_size * 2_f64).as_str()),
                    ("overflow", "scroll"),
                ])?;
            }

            // e.stop_propagation();
            Ok(())
        }) as Box<dyn FnMut(Event) -> Result<(), JsValue>>
    } else {
        Box::new(move |_: Event| -> Result<(), JsValue> { Ok(()) })
            as Box<dyn FnMut(Event) -> Result<(), JsValue>>
    };
    let f = Closure::wrap(f);
    flower_kind_elm.set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    Ok(())
}

fn cursor_set(s: &Singleton) -> Result<(), JsValue> {
    s.editor.class_list().remove(&js_sys::Array::from_iter(
        vec![
            "pointer-cursor",
            "seed-cursor",
            "can-cursor",
            "shovel-cursor",
        ]
        .into_iter()
        .map(|s| JsValue::from(s))
        .collect::<Vec<_>>()
        .iter(),
    ))?;

    let cursor = if s.inp_mode_menu.checked() {
        "pointer-cursor"
    } else if s.inp_mode_seed.checked() {
        "seed-cursor"
    } else if s.inp_mode_hybrid.checked() {
        "can-cursor"
    } else if s.inp_mode_remove.checked() {
        "shovel-cursor"
    } else {
        ""
    };

    s.editor.class_list().add(&js_sys::Array::from_iter(
        vec![&JsValue::from(cursor)].into_iter(),
    ))?;

    Ok(())
}

fn init_pallet(s: &Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let pallet_toggle = Rc::new(
        s.document
            .get_element_by_id("pallet-toggle")
            .expect("Error!: pallet-toggle")
            .dyn_into::<HtmlElement>()
            .unwrap(),
    );

    let f_s = s_rc.clone();
    let f_toggle = pallet_toggle.clone();
    let f = Closure::wrap(Box::new(move |e: Event| -> Result<(), JsValue> {
        let s = f_s.borrow();
        let state = s.pallet.style().get_property_value("display")?;
        let next = match state.as_str() {
            "block" => {
                f_toggle.set_inner_html("パレット▼");
                "none"
            }
            _ => {
                f_toggle.set_inner_html("パレット▲");
                set_pallet_contents(&s)?;
                "block" // 初回は値が得られないのでこう書いておくと都合がいい
            }
        };
        s.pallet.style().set_property("display", next)?;
        // console_log!("pallet {:?}", e.target());
        e.stop_propagation();
        Ok(())
    }) as Box<dyn FnMut(Event) -> Result<(), JsValue>>);
    pallet_toggle.add_event_listener_with_callback("click", f.as_ref().unchecked_ref())?;
    f.forget();

    for flower in Flower::get_all_flowers().into_iter() {
        let line = s
            .document
            .create_element("div")?
            .dyn_into::<HtmlElement>()
            .unwrap();
        let square = s
            .document
            .create_element("div")?
            .dyn_into::<HtmlElement>()
            .unwrap();

        square.class_list().add(&js_sys::Array::from_iter(
            vec![&JsValue::from("sample-square")].into_iter(),
        ))?;
        square.set_css_styles(vec![
            ("background-color", flower.get_color().to_string().as_str()),
            // ("display", "inline"),
        ])?;

        let name_label = s
            .document
            .create_element("span")?
            .dyn_into::<HtmlElement>()
            .unwrap();
        name_label.set_inner_html(format!("{} {}", flower.to_string(), flower.get_name()).as_str());
        line.append_child(&square)?;
        line.append_child(&name_label)?;

        let mut c_list = vec![
            "pallet-item".to_string(),
            "clickable".to_string(),
            flower.kind.to_string(),
            flower.get_color().to_string(),
        ];
        if flower.is_seed_flower() {
            c_list.push("seed".to_string());
        }

        line.class_list().add(&js_sys::Array::from_iter(
            c_list.into_iter().map(|c| JsValue::from(c)),
        ))?;

        line.set_css_styles(vec![("display", "flex")])?;

        let f_s = s_rc.clone();
        let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
            let mut s = f_s.borrow_mut();
            update_seed_form(flower, &mut s)?;
            s.pallet.style().set_property("display", "none")?;
            Ok(())
        }) as Box<dyn FnMut() -> Result<(), JsValue>>);
        line.set_onclick(Some(f.as_ref().unchecked_ref()));
        f.forget();

        s.pallet.append_child(&line)?;
    }

    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        set_pallet_contents(&s)?;

        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    s.pallet_inp_color
        .set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    Ok(())
}

fn set_pallet_contents(s: &Singleton) -> Result<(), JsValue> {
    let lines = s.pallet.query_selector_all(".pallet-item")?;

    for i in 0..lines.length() {
        let line = lines
            .get(i as u32)
            .expect("Error!: pallet-lines")
            .dyn_into::<HtmlElement>()
            .unwrap();
        let flag = line.class_list().contains(s.inp_kind.value().as_str());
        let color = s.pallet_inp_color.value();
        let flag = flag && (color == "all" || line.class_list().contains(color.as_str()));
        let prop = if flag { "flex" } else { "none" };
        line.style().set_property("display", prop)?;
    }

    Ok(())
}

fn hide_overlay(s: &Singleton) -> Result<(), JsValue> {
    s.overlay.style().set_property("display", "none")?;
    let hide_targets = s.document.query_selector_all("#modal > :not(#batsu)")?;

    for i in 0..hide_targets.length() {
        let item = hide_targets
            .get(i)
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        item.style().set_property("display", "none")?;
    }

    sns_hide(s)?;

    Ok(())
}

fn sns_hide(s: &Singleton) -> Result<(), JsValue> {
    let hide_targets = s.document.query_selector_all(".sns")?;
    for i in 0..hide_targets.length() {
        let item = hide_targets
            .get(i)
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        item.class_list().add(&js_sys::Array::from_iter(
            vec![&JsValue::from("hidden")].into_iter(),
        ))?;
    }
    Ok(())
}

fn init_overlay(s: &Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move |e: MouseEvent| -> Result<(), JsValue> {
        let s = f_s.borrow();
        let state = s.overlay.style().get_property_value("display")?;
        if state.as_str() != "none" {
            hide_overlay(&s)?;
        }
        e.stop_propagation();
        Ok(())
    }) as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);
    s.overlay.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    Ok(())
}

fn init_all_delete(s: &Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let all_delete = s
        .document
        .get_element_by_id("all-delete")
        .expect("Error!: all_delete")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let delete_modal = s
        .document
        .get_element_by_id("delete-modal")
        .expect("Error!: delete_modal")
        .dyn_into::<HtmlElement>()
        .unwrap();

    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        delete_modal.style().set_property("display", "block")?;
        s.overlay.style().set_property("display", "block")?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    all_delete.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    let delete_yes = s
        .document
        .get_element_by_id("delete-yes")
        .expect("Error!: delete_yes")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        do_all_delete(&s)?;
        hide_overlay(&s)?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    delete_yes.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();
    let delete_no = s
        .document
        .get_element_by_id("delete-no")
        .expect("Error!: delete_no")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        hide_overlay(&s)?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    delete_no.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    Ok(())
}

fn do_all_delete(s: &Singleton) -> Result<(), JsValue> {
    // とりあえずリロードだけでうまくいくということにしておく...そのままの可能性も大
    s.window.location().set_href("/")?;
    Ok(())
}

// use std::str::from_utf8;

/*
type CellBook = HashMap<(usize, usize), CanvasCell>;
type HybridBook = HashMap<HT, HybridLine>;
 */

fn init_save_mode(s: &Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let save_menu_button = s
        .document
        .get_element_by_id("save-menu-button")
        .expect("Error!: save-menu-button")
        .dyn_into::<HtmlElement>()
        .unwrap();

    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        popup_save_menu(&s)?;
        // popup_cookie_confirm(&s)?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    save_menu_button.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    init_save_menu(&s, &s_rc)?;
    init_cookie_confirm(&s, &s_rc)?;

    Ok(())
}

fn init_cookie_confirm(s: &Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let cookie_yes = s
        .document
        .get_element_by_id("cookie-yes")
        .expect("Error!: cookie_yes")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let cookie_modal = s
        .document
        .get_element_by_id("cookie-modal")
        .expect("Error!: cookie_modal")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move |e: MouseEvent| -> Result<(), JsValue> {
        let s = f_s.borrow();
        // hide_overlay(&s)?;
        cookie_modal.style().set_property("display", "none")?;
        data_save_to_cookie(&s)?;
        e.stop_propagation();
        Ok(())
    }) as Box<dyn FnMut(MouseEvent) -> Result<(), JsValue>>);

    cookie_yes.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    /*
    let cookie_no = s
        .document
        .get_element_by_id("cookie-no")
        .expect("Error!: cookie_no")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        hide_overlay(&s)?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    cookie_no.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();
     */

    Ok(())
}

fn popup_cookie_confirm(s: &Singleton) -> Result<(), JsValue> {
    sns_hide(s)?;
    s.overlay.style().set_property("display", "block")?;
    let cookie_modal = s
        .document
        .get_element_by_id("cookie-modal")
        .expect("Error!: cookie_modal")
        .dyn_into::<HtmlElement>()
        .unwrap();

    cookie_modal.style().set_property("display", "block")?;

    Ok(())
}

fn popup_alert(s: &Singleton, mes: &str, icon: &str) -> Result<(), JsValue> {
    sns_hide(s)?;
    s.overlay.style().set_property("display", "block")?;
    let alert_message = s
        .document
        .get_element_by_id("alert-message")
        .expect("Error!: alert_message")
        .dyn_into::<HtmlElement>()
        .unwrap();
    alert_message.set_inner_html(mes);
    let alert_icon = s
        .document
        .get_element_by_id("alert-icon")
        .expect("Error!: alert_icon")
        .dyn_into::<HtmlImageElement>()
        .unwrap();
    alert_icon.set_src(icon);
    let alert_modal = s
        .document
        .get_element_by_id("alert-modal")
        .expect("Error!: alert_modal")
        .dyn_into::<HtmlElement>()
        .unwrap();

    alert_modal.style().set_property("display", "block")?;

    Ok(())
}

fn init_save_menu(s: &Singleton, s_rc: &Rc<RefCell<Singleton>>) -> Result<(), JsValue> {
    let save_filename = s
        .document
        .get_element_by_id("save-filename")
        .expect("Error!: save-filename")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        s.inp_filename.set_value(&save_filename.value());
        s.canvas_book.save()?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    let save_filename = s
        .document
        .get_element_by_id("save-filename")
        .expect("Error!: save-filename")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    save_filename.set_oninput(Some(f.as_ref().unchecked_ref()));
    f.forget();

    let save_button = s
        .document
        .get_element_by_id("save-button")
        .expect("Error!: save-button")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let save_modal = s
        .document
        .get_element_by_id("save-modal")
        .expect("Error!: save_modal")
        .dyn_into::<HtmlElement>()
        .unwrap();

    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        save_modal.style().set_property("display", "none")?;
        if get_savedata()?.len() == 0 {
            popup_cookie_confirm(&s)?;
        } else {
            data_save_to_cookie(&s)?;
        }
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    save_button.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    let load = s
        .document
        .get_element_by_id("load")
        .expect("Error!: load")
        .dyn_into::<HtmlSelectElement>()
        .unwrap();
    load.set_inner_html("");

    let data = get_savedata()?;
    for d in data.into_iter() {
        let option = s
            .document
            .create_element("option")?
            .dyn_into::<HtmlOptionElement>()
            .unwrap();
        option.set_value(&d);
        option.set_text(CanvasBook::get_filename_from_hash(&d)?.as_str());
        load.add_with_html_option_element(&option)?;
    }

    let load_button = s
        .document
        .get_element_by_id("load-button")
        .expect("Error!: load_button")
        .dyn_into::<HtmlElement>()
        .unwrap();

    let f_s = s_rc.clone();
    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        let s = f_s.borrow();
        let v = load.value();
        let u = v.as_str();
        if u != "" {
            s.window.location().set_hash(u)?;
        }
        hide_overlay(&s)?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    load_button.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    let clipboard_copy = s
        .document
        .get_element_by_id("clipboard-copy")
        .expect("Error!: clipboard_copy")
        .dyn_into::<HtmlElement>()
        .unwrap();
    let share_url = s
        .document
        .get_element_by_id("share-url")
        .expect("Error!: share_url")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let hd = window()
        .expect("Error!: window")
        .document()
        .expect("Error!: document")
        .dyn_into::<HtmlDocument>()
        .unwrap();

    let f = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        share_url.select();
        hd.exec_command("copy")?;
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    clipboard_copy.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();

    Ok(())
}

fn popup_save_menu(s: &Singleton) -> Result<(), JsValue> {
    s.overlay.style().set_property("display", "block")?;
    let save_modal = s
        .document
        .get_element_by_id("save-modal")
        .expect("Error!: save_modal")
        .dyn_into::<HtmlElement>()
        .unwrap();

    save_modal.style().set_property("display", "block")?;

    let save_filename = s
        .document
        .get_element_by_id("save-filename")
        .expect("Error!: save-filename")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    save_filename.set_value(&s.inp_filename.value());

    let load = s
        .document
        .get_element_by_id("load")
        .expect("Error!: load")
        .dyn_into::<HtmlSelectElement>()
        .unwrap();
    load.set_inner_html("");

    let data = get_savedata()?;
    for d in data.into_iter() {
        let option = s
            .document
            .create_element("option")?
            .dyn_into::<HtmlOptionElement>()
            .unwrap();
        option.set_value(&d);
        option.set_text(CanvasBook::get_filename_from_hash(&d)?.as_str());
        load.add_with_html_option_element(&option)?;
    }

    let name = CanvasBook::get_filename_from_hash(&s.canvas_book.get_hash())?;
    let href = s.window.location().href()?;

    let share_url = s
        .document
        .get_element_by_id("share-url")
        .expect("Error!: share_url")
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    share_url.set_value(&href);

    set_tweet_button(href.as_str(), name.as_str());

    let twitter = s
        .document
        .get_element_by_id("twitter-entity")
        .expect("Error!: twitter")
        .dyn_into::<HtmlElement>()
        .unwrap();

    twitter.class_list().remove(&js_sys::Array::from_iter(
        vec![&JsValue::from("hidden")].into_iter(),
    ))?;

    let twitter_slot = s
        .document
        .get_element_by_id("twitter-slot")
        .expect("Error!: slot")
        .dyn_into::<HtmlElement>()
        .unwrap();

    let dom_rect = twitter_slot.get_bounding_client_rect();
    console_log!("{} {}", dom_rect.x(), dom_rect.y());

    twitter.set_css_styles(vec![
        ("left", format!("{}px", dom_rect.x()).as_str()),
        ("top", format!("{}px", dom_rect.y()).as_str()),
    ])?;

    Ok(())
}

fn get_savedata() -> Result<Vec<String>, JsValue> {
    let hd = window()
        .expect("Error!: window")
        .document()
        .expect("Error!: document")
        .dyn_into::<HtmlDocument>()
        .unwrap();
    let cookie = hd.cookie()?;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"data\d+=([^;]+)").unwrap();
    }
    let res = RE
        .captures_iter(cookie.as_str())
        .map(|caps| -> Result<String, JsValue> {
            let t = js_sys::decode_uri_component(&caps[1])?;
            Ok(format!("{:?}", t).replace("\"", ""))
        })
        .collect::<Result<Vec<String>, JsValue>>()?;
    Ok(res)
}

fn data_save_to_cookie(s: &Singleton) -> Result<(), JsValue> {
    let hd = window()
        .expect("Error!: window")
        .document()
        .expect("Error!: document")
        .dyn_into::<HtmlDocument>()
        .unwrap();
    let len = get_savedata()?.len();
    let hash = s.canvas_book.get_hash();
    let t = format!("{:?}", js_sys::encode_uri_component(&hash));
    let hash = t.replace("\"", "");
    let cookie = format!("data{}={}", len + 1, hash);
    hd.set_cookie(&cookie)?;
    let max_age = format!("max-age={}", 60 * 60 * 24 * 365 * 10);
    hd.set_cookie(&max_age)?;

    popup_alert(s, "保存しました", "/book.png")?;

    Ok(())
}
