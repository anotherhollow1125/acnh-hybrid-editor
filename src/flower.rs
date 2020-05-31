// #[macro_use]
// extern crate lazy_static;
extern crate regex;

use std::fmt;
use std::str::FromStr;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum Gene {
    G00,
    G01,
    G11,
}

impl fmt::Display for Gene {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                G00 => "00",
                G01 => "01",
                G11 => "11",
            }
        )
    }
}

use std::ops::{Div, Mul};

use Gene::*;

impl From<&str> for Gene {
    fn from(s: &str) -> Self {
        match s {
            "00" | "0" => G00,
            "01" | "1" => G01,
            "11" | "2" => G11,
            _ => G00,
        }
    }
}

impl Mul for Gene {
    type Output = Vec<(usize, Gene)>;

    fn mul(self, other: Gene) -> Vec<(usize, Gene)> {
        match self {
            G00 => match other {
                G00 => vec![(4, G00)],
                G01 => vec![(2, G01), (2, G00)],
                G11 => vec![(4, G01)],
            },
            G01 => match other {
                G00 => vec![(2, G01), (2, G00)],
                G01 => vec![(1, G00), (2, G01), (1, G11)],
                G11 => vec![(2, G01), (2, G11)],
            },
            G11 => match other {
                G00 => vec![(4, G01)],
                G01 => vec![(2, G01), (2, G11)],
                G11 => vec![(4, G11)],
            },
        }
    }
}

impl Div for Gene {
    type Output = Vec<(usize, Gene)>;

    fn div(self, other: Self) -> Vec<(usize, Gene)> {
        match self {
            G00 => match other {
                G00 => vec![(2, G01), (4, G00)],
                G01 => vec![(1, G01), (2, G00)],
                G11 => vec![],
            },
            G01 => match other {
                G00 => vec![(4, G11), (2, G01)],
                G01 => vec![(2, G00), (2, G01), (2, G11)],
                G11 => vec![(4, G00), (2, G01)],
            },
            G11 => match other {
                G00 => vec![],
                G01 => vec![(1, G01), (2, G11)],
                G11 => vec![(4, G11), (2, G01)],
            },
        }
    }
}

use std::collections::BinaryHeap;

// R, Y, W, B/S
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct FG(Gene, Gene, Gene, Gene);
// type FlowerGene = (Gene, Gene, Gene, Gene);

impl Mul for FG {
    type Output = Vec<(usize, Self)>;

    fn mul(self, other: Self) -> Vec<(usize, Self)> {
        let ws = self.0 * other.0;
        let ys = self.1 * other.1;
        let rs = self.2 * other.2;
        let cs = self.3 * other.3;
        let mut heap = BinaryHeap::new();

        for &w in ws.iter() {
            for &y in ys.iter() {
                for &r in rs.iter() {
                    for &c in cs.iter() {
                        let p = w.0 * y.0 * r.0 * c.0;
                        let child = FG(w.1, y.1, r.1, c.1);

                        heap.push((p, child));
                    }
                }
            }
        }

        heap.into_sorted_vec()
    }
}

impl Div for FG {
    type Output = Vec<(usize, Self)>;

    fn div(self, other: Self) -> Vec<(usize, Self)> {
        let ws = self.0 / other.0;
        let ys = self.1 / other.1;
        let rs = self.2 / other.2;
        let cs = self.3 / other.3;
        let mut heap = BinaryHeap::new();

        for &w in ws.iter() {
            for &y in ys.iter() {
                for &r in rs.iter() {
                    for &c in cs.iter() {
                        let p = w.0 * y.0 * r.0 * c.0;
                        let child = FG(w.1, y.1, r.1, c.1);

                        heap.push((p, child));
                    }
                }
            }
        }

        heap.into_sorted_vec()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Color {
    Red,
    White,
    Yellow,
    Pink,
    Orange,
    Blue,
    Purple,
    Black,
    Green,
    // Gold,
}

use Color::*;

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self).to_string().to_lowercase();
        write!(f, "{}", s)
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "red" => Ok(Red),
            "white" => Ok(White),
            "yellow" => Ok(Yellow),
            "pink" => Ok(Pink),
            "orange" => Ok(Orange),
            "blue" => Ok(Blue),
            "purple" => Ok(Purple),
            "black" => Ok(Black),
            "green" => Ok(Green),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum FlowerKind {
    Rose,
    Cosmos,
    Tulip,
    Pansy,
    Lily,
    Hyacinth,
    Anemone,
    Mum,
}

use FlowerKind::*;

impl fmt::Display for FlowerKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self).to_lowercase();
        write!(f, "{}", s)
    }
}

impl FromStr for FlowerKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rose" => Ok(Rose),
            "cosmos" => Ok(Cosmos),
            "tulip" => Ok(Tulip),
            "pansy" => Ok(Pansy),
            "lily" => Ok(Lily),
            "hyacinth" => Ok(Hyacinth),
            "anemone" => Ok(Anemone),
            "mum" => Ok(Mum),
            _ => Err(()),
        }
    }
}

impl FlowerKind {
    pub fn vec() -> Vec<Self> {
        vec![Rose, Cosmos, Tulip, Pansy, Lily, Hyacinth, Anemone, Mum]
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Flower {
    pub kind: FlowerKind,
    pub genome: FG,
}

pub struct SeedFlowers {
    pub red: Flower,
    pub yellow: Flower,
    pub white: Flower,
}

impl SeedFlowers {
    pub fn to_vec(&self) -> Vec<Flower> {
        vec![self.red, self.yellow, self.white]
    }
}

impl fmt::Display for Flower {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Flower {
                kind: Rose,
                genome: g,
            } => format!("{}-{}-{}-{}", g.0, g.1, g.2, g.3).to_string(),
            Flower { kind: _, genome: g } => format!("{}-{}-{}", g.0, g.1, g.2).to_string(),
        };
        write!(f, "{}", s)
    }
}

use regex::Regex;

impl Flower {
    pub fn new(kind: FlowerKind, genome: FG) -> Self {
        Flower {
            kind,
            genome: match kind {
                Rose => genome,
                _ => FG(genome.0, genome.1, genome.2, G00),
            },
        }
    }

    pub fn strs2flower(kind_str: &str, genome_str: &str) -> Option<Flower> {
        let kind = match kind_str.to_lowercase().as_str() {
            "rose" => Rose,
            "cosmos" => Cosmos,
            "tulip" => Tulip,
            "pansy" => Pansy,
            "lily" => Lily,
            "hyacinth" => Hyacinth,
            "anemone" => Anemone,
            "mum" => Mum,
            _ => return None,
        };
        lazy_static! {
            static ref RE: Regex = Regex::new(r"[^RrYyWwBbSs012]").unwrap();
        }
        let genome_str = RE.replace_all(genome_str, "");
        lazy_static! {
            static ref RE2: Regex = Regex::new(r"[RrYyWwBbSs]").unwrap();
        }
        let genome = if RE2.find(&genome_str).is_none() {
            let mut genome_str = genome_str.to_string();
            let p = if genome_str.len() <= 4 {
                if genome_str.len() < 3 {
                    return None;
                }
                1
            } else {
                if genome_str.len() <= 5 {
                    return None;
                }
                2
            };
            let mut rest = genome_str.split_off(p);
            let g1 = Gene::from(genome_str.as_str());
            let mut rest2 = rest.split_off(p);
            let g2 = Gene::from(rest.as_str());
            let mut rest3 = rest2.split_off(p);
            let g3 = Gene::from(rest2.as_str());
            let g4 = if rest3.len() >= p {
                let _ = rest3.split_off(p);
                Gene::from(rest3.as_str())
            } else {
                G00
            };
            FG(g1, g2, g3, g4)
        } else {
            let genome_str = genome_str.replace("S", "B");
            let v = ["R", "Y", "W", "B"]
                .into_iter()
                .map(|c| {
                    if genome_str.contains(c.repeat(2).as_str()) {
                        G11
                    } else if genome_str.contains(c) {
                        G01
                    } else {
                        G00
                    }
                })
                .collect::<Vec<_>>();
            FG(v[0], v[1], v[2], v[3])
        };

        Some(Flower::new(kind, genome))
    }

    pub fn get_name(&self) -> String {
        format!(
            "{}_{}{}",
            self.kind,
            self.get_color(),
            if self.is_seed_flower() { "_seed" } else { "" }
        )
    }

    pub fn hybrid_with(&self, other: &Self) -> Option<Vec<(usize, Self)>> {
        if self.kind != other.kind {
            return None;
        }
        Some(
            (self.genome * other.genome)
                .into_iter()
                .map(|t| (t.0, Flower::new(self.kind, t.1)))
                .collect::<Vec<_>>(),
        )
    }

    pub fn get_other_parents(&self, father: &Self) -> Option<Vec<(usize, Self)>> {
        if self.kind != father.kind {
            return None;
        }
        Some(
            (self.genome / father.genome)
                .into_iter()
                .map(|t| (t.0, Flower::new(self.kind, t.1)))
                .collect::<Vec<_>>(),
        )
    }

    pub fn get_chance_from(&self, father: &Self, mother: &Self) -> usize {
        let list_w = father.hybrid_with(mother);
        let list = match list_w {
            Some(v) => v,
            None => return 0,
        };

        for (p, flower) in list.into_iter() {
            if self.kind != flower.kind {
                break;
            }

            if self.genome == flower.genome {
                return p;
            }
        }

        0
    }

    pub fn is_seed_flower(&self) -> bool {
        Self::get_seed_flowers(self.kind).to_vec().contains(&self)
    }

    pub fn get_seed_flowers(kind: FlowerKind) -> SeedFlowers {
        let genomes = match kind {
            Rose => vec![
                FG(G11, G00, G00, G01),
                FG(G00, G11, G00, G00),
                FG(G00, G00, G01, G00),
            ],
            Cosmos => vec![
                FG(G11, G00, G00, G00),
                FG(G00, G11, G01, G00),
                FG(G00, G00, G01, G00),
            ],
            Tulip => vec![
                FG(G11, G00, G01, G00),
                FG(G00, G11, G00, G00),
                FG(G00, G00, G01, G00),
            ],
            Pansy => vec![
                FG(G11, G00, G00, G00),
                FG(G00, G11, G00, G00),
                FG(G00, G00, G01, G00),
            ],
            Lily => vec![
                FG(G11, G00, G01, G00),
                FG(G00, G11, G00, G00),
                FG(G00, G00, G11, G00),
            ],
            Hyacinth => vec![
                FG(G11, G00, G01, G00),
                FG(G00, G11, G00, G00),
                FG(G00, G00, G01, G00),
            ],
            Anemone => vec![
                FG(G11, G00, G00, G00),
                FG(G00, G11, G00, G00),
                FG(G00, G00, G01, G00),
            ],
            Mum => vec![
                FG(G11, G00, G00, G00),
                FG(G00, G11, G00, G00),
                FG(G00, G00, G01, G00),
            ],
        };

        SeedFlowers {
            red: Flower::new(kind, genomes[0]),
            yellow: Flower::new(kind, genomes[1]),
            white: Flower::new(kind, genomes[2]),
        }
    }

    pub fn get_all_flowers() -> Vec<Flower> {
        let mut ans = Vec::with_capacity(270);
        for gr in vec![G00, G01, G11].into_iter() {
            for gy in vec![G00, G01, G11].into_iter() {
                for gw in vec![G00, G01, G11].into_iter() {
                    // Rose
                    for gs in vec![G00, G01, G11].into_iter() {
                        ans.push(Flower::new(Rose, FG(gr, gy, gw, gs)));
                    }

                    // Others
                    for kind in vec![Cosmos, Tulip, Pansy, Lily, Hyacinth, Anemone, Mum].into_iter()
                    {
                        ans.push(Flower::new(kind, FG(gr, gy, gw, G00)));
                    }
                }
            }
        }
        ans
    }

    pub fn get_color(&self) -> Color {
        match self.kind {
            Rose => match self.genome {
                FG(G00, G00, G00, G00) => White,
                FG(G00, G00, G00, G01) => White,
                FG(G00, G00, G00, G11) => White,
                FG(G00, G00, G01, G00) => White,
                FG(G00, G00, G01, G01) => White,
                FG(G00, G00, G01, G11) => White,
                FG(G00, G00, G11, G00) => Purple,
                FG(G00, G00, G11, G01) => Purple,
                FG(G00, G00, G11, G11) => Purple,
                FG(G00, G01, G00, G00) => Yellow,
                FG(G00, G01, G00, G01) => Yellow,
                FG(G00, G01, G00, G11) => Yellow,
                FG(G00, G01, G01, G00) => White,
                FG(G00, G01, G01, G01) => White,
                FG(G00, G01, G01, G11) => White,
                FG(G00, G01, G11, G00) => Purple,
                FG(G00, G01, G11, G01) => Purple,
                FG(G00, G01, G11, G11) => Purple,
                FG(G00, G11, G00, G00) => Yellow,
                FG(G00, G11, G00, G01) => Yellow,
                FG(G00, G11, G00, G11) => Yellow,
                FG(G00, G11, G01, G00) => Yellow,
                FG(G00, G11, G01, G01) => Yellow,
                FG(G00, G11, G01, G11) => Yellow,
                FG(G00, G11, G11, G00) => White,
                FG(G00, G11, G11, G01) => White,
                FG(G00, G11, G11, G11) => White,
                FG(G01, G00, G00, G00) => Red,
                FG(G01, G00, G00, G01) => Pink,
                FG(G01, G00, G00, G11) => White,
                FG(G01, G00, G01, G00) => Red,
                FG(G01, G00, G01, G01) => Pink,
                FG(G01, G00, G01, G11) => White,
                FG(G01, G00, G11, G00) => Red,
                FG(G01, G00, G11, G01) => Pink,
                FG(G01, G00, G11, G11) => Purple,
                FG(G01, G01, G00, G00) => Orange,
                FG(G01, G01, G00, G01) => Yellow,
                FG(G01, G01, G00, G11) => Yellow,
                FG(G01, G01, G01, G00) => Red,
                FG(G01, G01, G01, G01) => Pink,
                FG(G01, G01, G01, G11) => White,
                FG(G01, G01, G11, G00) => Red,
                FG(G01, G01, G11, G01) => Pink,
                FG(G01, G01, G11, G11) => Purple,
                FG(G01, G11, G00, G00) => Orange,
                FG(G01, G11, G00, G01) => Yellow,
                FG(G01, G11, G00, G11) => Yellow,
                FG(G01, G11, G01, G00) => Orange,
                FG(G01, G11, G01, G01) => Yellow,
                FG(G01, G11, G01, G11) => Yellow,
                FG(G01, G11, G11, G00) => Red,
                FG(G01, G11, G11, G01) => Pink,
                FG(G01, G11, G11, G11) => White,
                FG(G11, G00, G00, G00) => Black,
                FG(G11, G00, G00, G01) => Red,
                FG(G11, G00, G00, G11) => Pink,
                FG(G11, G00, G01, G00) => Black,
                FG(G11, G00, G01, G01) => Red,
                FG(G11, G00, G01, G11) => Pink,
                FG(G11, G00, G11, G00) => Black,
                FG(G11, G00, G11, G01) => Red,
                FG(G11, G00, G11, G11) => Pink,
                FG(G11, G01, G00, G00) => Orange,
                FG(G11, G01, G00, G01) => Orange,
                FG(G11, G01, G00, G11) => Yellow,
                FG(G11, G01, G01, G00) => Red,
                FG(G11, G01, G01, G01) => Red,
                FG(G11, G01, G01, G11) => White,
                FG(G11, G01, G11, G00) => Black,
                FG(G11, G01, G11, G01) => Red,
                FG(G11, G01, G11, G11) => Purple,
                FG(G11, G11, G00, G00) => Orange,
                FG(G11, G11, G00, G01) => Orange,
                FG(G11, G11, G00, G11) => Yellow,
                FG(G11, G11, G01, G00) => Orange,
                FG(G11, G11, G01, G01) => Orange,
                FG(G11, G11, G01, G11) => Yellow,
                FG(G11, G11, G11, G00) => Blue,
                FG(G11, G11, G11, G01) => Red,
                FG(G11, G11, G11, G11) => White,
            },
            Cosmos => match self.genome {
                FG(G00, G00, G00, _) => White,
                FG(G00, G00, G01, _) => White,
                FG(G00, G00, G11, _) => White,
                FG(G00, G01, G00, _) => Yellow,
                FG(G00, G01, G01, _) => Yellow,
                FG(G00, G01, G11, _) => White,
                FG(G00, G11, G00, _) => Yellow,
                FG(G00, G11, G01, _) => Yellow,
                FG(G00, G11, G11, _) => Yellow,
                FG(G01, G00, G00, _) => Pink,
                FG(G01, G00, G01, _) => Pink,
                FG(G01, G00, G11, _) => Pink,
                FG(G01, G01, G00, _) => Orange,
                FG(G01, G01, G01, _) => Orange,
                FG(G01, G01, G11, _) => Pink,
                FG(G01, G11, G00, _) => Orange,
                FG(G01, G11, G01, _) => Orange,
                FG(G01, G11, G11, _) => Orange,
                FG(G11, G00, G00, _) => Red,
                FG(G11, G00, G01, _) => Red,
                FG(G11, G00, G11, _) => Red,
                FG(G11, G01, G00, _) => Orange,
                FG(G11, G01, G01, _) => Orange,
                FG(G11, G01, G11, _) => Red,
                FG(G11, G11, G00, _) => Black,
                FG(G11, G11, G01, _) => Black,
                FG(G11, G11, G11, _) => Red,
            },
            Tulip => match self.genome {
                FG(G00, G00, G00, _) => White,
                FG(G00, G00, G01, _) => White,
                FG(G00, G00, G11, _) => White,
                FG(G00, G01, G00, _) => Yellow,
                FG(G00, G01, G01, _) => Yellow,
                FG(G00, G01, G11, _) => White,
                FG(G00, G11, G00, _) => Yellow,
                FG(G00, G11, G01, _) => Yellow,
                FG(G00, G11, G11, _) => Yellow,
                FG(G01, G00, G00, _) => Red,
                FG(G01, G00, G01, _) => Pink,
                FG(G01, G00, G11, _) => White,
                FG(G01, G01, G00, _) => Orange,
                FG(G01, G01, G01, _) => Yellow,
                FG(G01, G01, G11, _) => Yellow,
                FG(G01, G11, G00, _) => Orange,
                FG(G01, G11, G01, _) => Yellow,
                FG(G01, G11, G11, _) => Yellow,
                FG(G11, G00, G00, _) => Black,
                FG(G11, G00, G01, _) => Red,
                FG(G11, G00, G11, _) => Red,
                FG(G11, G01, G00, _) => Black,
                FG(G11, G01, G01, _) => Red,
                FG(G11, G01, G11, _) => Red,
                FG(G11, G11, G00, _) => Purple,
                FG(G11, G11, G01, _) => Purple,
                FG(G11, G11, G11, _) => Purple,
            },
            Pansy => match self.genome {
                FG(G00, G00, G00, _) => White,
                FG(G00, G00, G01, _) => White,
                FG(G00, G00, G11, _) => Blue,
                FG(G00, G01, G00, _) => Yellow,
                FG(G00, G01, G01, _) => Yellow,
                FG(G00, G01, G11, _) => Blue,
                FG(G00, G11, G00, _) => Yellow,
                FG(G00, G11, G01, _) => Yellow,
                FG(G00, G11, G11, _) => Yellow,
                FG(G01, G00, G00, _) => Red,
                FG(G01, G00, G01, _) => Red,
                FG(G01, G00, G11, _) => Blue,
                FG(G01, G01, G00, _) => Orange,
                FG(G01, G01, G01, _) => Orange,
                FG(G01, G01, G11, _) => Orange,
                FG(G01, G11, G00, _) => Yellow,
                FG(G01, G11, G01, _) => Yellow,
                FG(G01, G11, G11, _) => Yellow,
                FG(G11, G00, G00, _) => Red,
                FG(G11, G00, G01, _) => Red,
                FG(G11, G00, G11, _) => Purple,
                FG(G11, G01, G00, _) => Red,
                FG(G11, G01, G01, _) => Red,
                FG(G11, G01, G11, _) => Purple,
                FG(G11, G11, G00, _) => Orange,
                FG(G11, G11, G01, _) => Orange,
                FG(G11, G11, G11, _) => Purple,
            },
            Lily => match self.genome {
                FG(G00, G00, G00, _) => White,
                FG(G00, G00, G01, _) => White,
                FG(G00, G00, G11, _) => White,
                FG(G00, G01, G00, _) => Yellow,
                FG(G00, G01, G01, _) => White,
                FG(G00, G01, G11, _) => White,
                FG(G00, G11, G00, _) => Yellow,
                FG(G00, G11, G01, _) => Yellow,
                FG(G00, G11, G11, _) => White,
                FG(G01, G00, G00, _) => Red,
                FG(G01, G00, G01, _) => Pink,
                FG(G01, G00, G11, _) => White,
                FG(G01, G01, G00, _) => Orange,
                FG(G01, G01, G01, _) => Yellow,
                FG(G01, G01, G11, _) => Yellow,
                FG(G01, G11, G00, _) => Orange,
                FG(G01, G11, G01, _) => Yellow,
                FG(G01, G11, G11, _) => Yellow,
                FG(G11, G00, G00, _) => Black,
                FG(G11, G00, G01, _) => Red,
                FG(G11, G00, G11, _) => Pink,
                FG(G11, G01, G00, _) => Black,
                FG(G11, G01, G01, _) => Red,
                FG(G11, G01, G11, _) => Pink,
                FG(G11, G11, G00, _) => Orange,
                FG(G11, G11, G01, _) => Orange,
                FG(G11, G11, G11, _) => White,
            },
            Hyacinth => match self.genome {
                FG(G00, G00, G00, _) => White,
                FG(G00, G00, G01, _) => White,
                FG(G00, G00, G11, _) => Blue,
                FG(G00, G01, G00, _) => Yellow,
                FG(G00, G01, G01, _) => Yellow,
                FG(G00, G01, G11, _) => White,
                FG(G00, G11, G00, _) => Yellow,
                FG(G00, G11, G01, _) => Yellow,
                FG(G00, G11, G11, _) => Yellow,
                FG(G01, G00, G00, _) => Red,
                FG(G01, G00, G01, _) => Pink,
                FG(G01, G00, G11, _) => White,
                FG(G01, G01, G00, _) => Orange,
                FG(G01, G01, G01, _) => Yellow,
                FG(G01, G01, G11, _) => Yellow,
                FG(G01, G11, G00, _) => Orange,
                FG(G01, G11, G01, _) => Yellow,
                FG(G01, G11, G11, _) => Yellow,
                FG(G11, G00, G00, _) => Red,
                FG(G11, G00, G01, _) => Red,
                FG(G11, G00, G11, _) => Red,
                FG(G11, G01, G00, _) => Blue,
                FG(G11, G01, G01, _) => Red,
                FG(G11, G01, G11, _) => Red,
                FG(G11, G11, G00, _) => Purple,
                FG(G11, G11, G01, _) => Purple,
                FG(G11, G11, G11, _) => Purple,
            },
            Anemone => match self.genome {
                FG(G00, G00, G00, _) => White,
                FG(G00, G00, G01, _) => White,
                FG(G00, G00, G11, _) => Blue,
                FG(G00, G01, G00, _) => Orange,
                FG(G00, G01, G01, _) => Orange,
                FG(G00, G01, G11, _) => Blue,
                FG(G00, G11, G00, _) => Orange,
                FG(G00, G11, G01, _) => Orange,
                FG(G00, G11, G11, _) => Orange,
                FG(G01, G00, G00, _) => Red,
                FG(G01, G00, G01, _) => Red,
                FG(G01, G00, G11, _) => Blue,
                FG(G01, G01, G00, _) => Pink,
                FG(G01, G01, G01, _) => Pink,
                FG(G01, G01, G11, _) => Pink,
                FG(G01, G11, G00, _) => Orange,
                FG(G01, G11, G01, _) => Orange,
                FG(G01, G11, G11, _) => Orange,
                FG(G11, G00, G00, _) => Red,
                FG(G11, G00, G01, _) => Red,
                FG(G11, G00, G11, _) => Purple,
                FG(G11, G01, G00, _) => Red,
                FG(G11, G01, G01, _) => Red,
                FG(G11, G01, G11, _) => Purple,
                FG(G11, G11, G00, _) => Pink,
                FG(G11, G11, G01, _) => Pink,
                FG(G11, G11, G11, _) => Purple,
            },
            Mum => match self.genome {
                FG(G00, G00, G00, _) => White,
                FG(G00, G00, G01, _) => White,
                FG(G00, G00, G11, _) => Purple,
                FG(G00, G01, G00, _) => Yellow,
                FG(G00, G01, G01, _) => Yellow,
                FG(G00, G01, G11, _) => White,
                FG(G00, G11, G00, _) => Yellow,
                FG(G00, G11, G01, _) => Yellow,
                FG(G00, G11, G11, _) => Yellow,
                FG(G01, G00, G00, _) => Pink,
                FG(G01, G00, G01, _) => Pink,
                FG(G01, G00, G11, _) => Pink,
                FG(G01, G01, G00, _) => Yellow,
                FG(G01, G01, G01, _) => Red,
                FG(G01, G01, G11, _) => Pink,
                FG(G01, G11, G00, _) => Purple,
                FG(G01, G11, G01, _) => Purple,
                FG(G01, G11, G11, _) => Purple,
                FG(G11, G00, G00, _) => Red,
                FG(G11, G00, G01, _) => Red,
                FG(G11, G00, G11, _) => Red,
                FG(G11, G01, G00, _) => Purple,
                FG(G11, G01, G01, _) => Purple,
                FG(G11, G01, G11, _) => Red,
                FG(G11, G11, G00, _) => Green,
                FG(G11, G11, G01, _) => Green,
                FG(G11, G11, G11, _) => Red,
            },
        }
    }
}
