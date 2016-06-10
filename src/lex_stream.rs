pub struct LexStream<TIterator: Iterator<Item=char>> {
    iterator: TIterator,
    lookahead_buf: Vec<char>,
}

impl<TIterator: Iterator<Item=char>> LexStream<TIterator> {
    pub fn new(iterator: TIterator) -> Self {
        LexStream {
            iterator: iterator,
            lookahead_buf: Vec::new(),
        }
    }
    
    pub fn read(&mut self) -> Option<char> {
        if self.lookahead_buf.len() > 0 {
            let c = self.lookahead_buf.remove(0);
            Some(c)
        } else {
            let result = self.iterator.next();
            result
        }
    }
    
    pub fn skip(&mut self, count: usize) {
        for _ in 0..count {
            if self.lookahead_buf.len() > 0 {
                self.lookahead_buf.remove(0);
            } else {
                self.iterator.next();
            }
        }
    }
    
    pub fn lookahead(&mut self, offset: usize) -> Option<char> {
        while self.lookahead_buf.len() < offset + 1 {
            match self.iterator.next() {
                Some(c) => self.lookahead_buf.push(c),
                None => return None,
            }
        }
        
        Some(self.lookahead_buf[offset])
    }
}
