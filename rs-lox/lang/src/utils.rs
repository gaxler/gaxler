pub struct Liner {
    line_bounds: Vec<u32>,
}

pub struct Span {
    pub line: u32,
    pub ch_in_line: u32,
    pub abs_ch: u32,
}

impl Liner {
    pub fn from_str(source: &str) -> Self {
        let line_bounds: Vec<_> = source
            .chars()
            .enumerate()
            .filter(|&(_, ch)| ch == '\n')
            .map(|(idx, _)| idx as u32)
            .collect();

        Self { line_bounds }
    }

    pub fn get_span(&self, abs_pos: usize) -> Span {
        let abs_pos = abs_pos as u32;
        let pos = match self.line_bounds.binary_search(&abs_pos) {
            Ok(pos) | Err(pos) => pos,
        };

        let ch_in_line = {
            if pos == 0 {
                abs_pos
            } else {
                abs_pos - self.line_bounds[pos - 1]
            }
        };

        Span {
            line: pos as u32 + 1,
            ch_in_line: ch_in_line,
            abs_ch: abs_pos,
        }
    }
}

impl From<&str> for Liner {
    fn from(s: &str) -> Self {
        Liner::from_str(s)
    }
}

pub fn cite_span(source: &str, st_pos: usize, en_pos: usize) -> String {
    let lin = Liner::from(source);
    let st = lin.get_span(st_pos);
    let en = lin.get_span(en_pos);

    let mut res = String::new();

    for (line_num_less_one, l) in source.lines().enumerate() {
        let lino = line_num_less_one as u32 + 1;
        let low = st.line;
        let high = en.line;


        if lino >= low && lino <= high {
            res.push_str(&format!(" {} | {}", lino, l));
            res.push_str("\n");
            
            res.push_str("   | ");

            if lino == low {
                for _ in 0..st.ch_in_line {
                    res.push_str(" ");
                }
                for _ in 0..(l.len() as u32 - st.ch_in_line) {
                    res.push_str("^");
                }
                res.push_str("\n");
            } else if lino == high {
                for _ in 0..en.ch_in_line {
                    res.push_str("^");
                }
                res.push_str("\n");
            } else {
                for _ in 0..l.len() {
                    res.push_str("^");
                }
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    const txt: &str = "hello\ni\nlove you\nwont you tell me your name";

    fn liner_helper(pos: usize, cor_line: u32, cor_ch: u32) {
        let liner: Liner = txt.into();
        let span = liner.get_span(pos);

        assert_eq!(span.line, cor_line);
        assert_eq!(span.ch_in_line, cor_ch);
    }
    #[test]
    fn line_anno() {
        use std::fs;

        let source = fs::read_to_string("/Users/gregoryaxler/Desktop/projects/gaxler/rs-lox/expr.lox").unwrap();
        let res = cite_span(&source, 32, 34);
        println!("{}", res);
    }
    // #[test]
    fn line_ann() {
        let lin: Liner = txt.into();
        let st = lin.get_span(7);
        let en = lin.get_span(20);

        for (line_num_less_one, l) in txt.lines().enumerate() {
            let lino = line_num_less_one as u32 + 1;
            let low = st.line;
            let high = en.line;

            if lino >= low && lino <= high {
                println!(" {} | {}", lino, l);
                print!("   | ");
                if lino == low {
                    for _ in 0..st.ch_in_line {
                        print!(" ");
                    }
                    for _ in 0..((l.len() + 1) as u32 - st.ch_in_line) {
                        print!("^");
                    }
                } else if lino == high {
                    for _ in 0..en.ch_in_line {
                        print!("^");
                    }
                } else {
                    for _ in 0..l.len() {
                        print!("^");
                    }
                }

                println!("");
            }
        }
    }
    #[test]
    fn liner_in_first() {
        liner_helper(2, 1u32, 2u32);
    }

    #[test]
    fn liner_in_mid() {
        liner_helper(10, 3u32, 3u32);
    }

    #[test]
    fn liner_in_last() {
        liner_helper(20, 4, 4);
    }
}
