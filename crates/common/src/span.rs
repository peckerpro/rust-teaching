use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub lo: BytePos,
    pub hi: BytePos,
}

impl Span {
    pub const DUMMY: Span = Span {
        lo: BytePos(0),
        hi: BytePos(0),
    };

    pub fn new(lo: BytePos, hi: BytePos) -> Self {
        Span { lo, hi }
    }

    pub fn to(self, other: Span) -> Span {
        Span {
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.lo == BytePos(0) && self.hi == BytePos(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytePos(pub u32);

impl fmt::Display for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl BytePos {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for BytePos {
    fn from(n: u32) -> Self {
        BytePos(n)
    }
}

#[derive(Debug)]
pub struct SourceFile {
    pub name: String,
    pub src: String,
    pub lines: Vec<usize>,
}

impl SourceFile {
    pub fn new(name: String, src: String) -> Self {
        let lines = std::iter::once(0)
            .chain(src.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        SourceFile { name, src, lines }
    }

    pub fn lookup_pos(&self, pos: BytePos) -> (usize, usize) {
        let pos = pos.to_usize();
        let line = self
            .lines
            .binary_search(&pos)
            .unwrap_or_else(|x| x.saturating_sub(1));
        let col = pos - self.lines[line];
        (line + 1, col + 1)
    }

    pub fn line_bounds(&self, line: usize) -> (usize, usize) {
        let line_start = self.lines[line];
        let line_end = self
            .lines
            .get(line + 1)
            .copied()
            .unwrap_or(self.src.len());
        (line_start, line_end)
    }
}
