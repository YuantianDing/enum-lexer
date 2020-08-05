use std::cmp;
use std::fmt;
use std::cell::RefCell;
use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    base: u32,
    token: RefCell<String>,
    chars: Peekable<Chars<'a>>,
}

impl<'a> Iterator for Cursor<'a> {
    type Item = char;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.chars.next();
        if let Some(ch) = ret {
            self.token.borrow_mut().push(ch);
        }
        ret
    }
}

impl<'a> Cursor<'a> {
    pub fn new_file(name: &str, src: &'a str) -> Cursor<'a> {
        SOURCE_MAP.with(|cm| {
            let mut cm = cm.borrow_mut();
            let span = cm.add_file(name, src);
            let base = span.lo;
            Cursor {
                base,
                token: RefCell::new(String::new()),
                chars: src.chars().peekable(),
            }
        })
    }

    #[inline(always)]
    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    #[inline(always)]
    pub fn leap(&mut self) {
        self.chars.next();
    }

    #[inline(always)]
    pub fn leap_until(&mut self, func: impl Fn(char) -> bool) {
        while let Some(&ch) = self.chars.peek() {
            if func(ch) {
                break;
            } else {
                self.leap();
            }
        }
    }
    
    #[inline(always)]
    pub fn get_token(&mut self) -> (String, Span) {
        let lo = self.base as u32;
        self.base += self.token.borrow().len() as u32;
        let hi = self.base as u32;
        (self.token.replace(String::new()), Span{ lo, hi } )
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

struct FileInfo {
    name: String,
    span: Span,
    lines: Vec<usize>,
}

impl FileInfo {
    fn offset_line_column(&self, offset: usize) -> LineColumn {
        assert!(self.span_within(Span {
            lo: offset as u32,
            hi: offset as u32
        }));
        let offset = offset - self.span.lo as usize;
        match self.lines.binary_search(&offset) {
            Ok(found) => LineColumn {
                line: found + 1,
                column: 0,
            },
            Err(idx) => LineColumn {
                line: idx,
                column: offset - self.lines[idx - 1],
            },
        }
    }

    fn span_within(&self, span: Span) -> bool {
        span.lo >= self.span.lo && span.hi <= self.span.hi
    }
}

/// Computes the offsets of each line in the given source string
/// and the total number of characters
fn lines_offsets(s: &str) -> (usize, Vec<usize>) {
    let mut lines = vec![0];
    let mut total = 0;

    for ch in s.chars() {
        total += 1;
        if ch == '\n' {
            lines.push(total);
        }
    }
    (total, lines)
}


struct SourceMap {
    files: Vec<FileInfo>,
}

impl SourceMap {
    fn next_start_pos(&self) -> u32 {
        // Add 1 so there's always space between files.
        //
        // We'll always have at least 1 file, as we initialize our files list
        // with a dummy file.
        self.files.last().unwrap().span.hi + 1
    }

    fn add_file(&mut self, name: &str, src: &str) -> Span {
        let (len, lines) = lines_offsets(src);
        let lo = self.next_start_pos();
        let span = Span {
            lo,
            hi: lo + (len as u32),
        };

        self.files.push(FileInfo {
            name: name.to_owned(),
            span,
            lines,
        });

        span
    }

    fn fileinfo(&self, span: Span) -> &FileInfo {
        for file in &self.files {
            if file.span_within(span) {
                return file;
            }
        }
        panic!("Invalid span with no related FileInfo!");
    }
}

thread_local! {
    static SOURCE_MAP: RefCell<SourceMap> = RefCell::new(SourceMap {
        // NOTE: We start with a single dummy file which all call_site() and
        // def_site() spans reference.
        files: vec![FileInfo {
            name: "<unspecified>".to_owned(),
            span: Span { lo: 0, hi: 0 },
            lines: vec![0],
        }],
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub(crate) lo: u32,
    pub(crate) hi: u32,
}

impl Span {

    pub fn source_name(&self) -> String {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(*self);
            fi.name.clone()
        })
    }

    pub fn start(&self) -> LineColumn {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(*self);
            fi.offset_line_column(self.lo as usize)
        })
    }

    pub fn end(&self) -> LineColumn {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(*self);
            fi.offset_line_column(self.hi as usize)
        })
    }

    pub fn join(&self, other: Span) -> Option<Span> {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            // If `other` is not within the same FileInfo as us, return None.
            if !cm.fileinfo(*self).span_within(other) {
                return None;
            }
            Some(Span {
                lo: cmp::min(self.lo, other.lo),
                hi: cmp::max(self.hi, other.hi),
            })
        })
    }

    pub fn first_byte(self) -> Self {
        Span {
            lo: self.lo,
            hi: cmp::min(self.lo.saturating_add(1), self.hi),
        }
    }

    pub fn last_byte(self) -> Self {
        Span {
            lo: cmp::max(self.hi.saturating_sub(1), self.lo),
            hi: self.hi,
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "bytes({}..{})", self.lo, self.hi);
    }
}

