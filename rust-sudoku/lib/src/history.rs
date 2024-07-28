use log::*;

pub struct History<T> {
    data: Vec<T>,
    cur: usize,
}

impl<T> History<T> {
    pub fn new(initial: T) -> Self {
        Self {
            data: vec![initial],
            cur: 0,
        }
    }

    pub fn current(&self) -> &T {
        &self.data[self.cur]
    }

    pub fn push(&mut self, value: T) {
        self.data.truncate(self.cur + 1);
        self.data.push(value);
        self.cur += 1;
    }

    pub fn undo(&mut self) {
        match self.cur {
            0 => {
                trace!("can't undo, already at beginning of history")
            }
            cur => {
                self.cur = cur - 1;
            }
        };
    }

    pub fn redo(&mut self) {
        match self.cur {
            cur if cur == self.data.len() - 1 => {
                trace!("can't redo, already at the end of history")
            }
            cur => {
                self.cur = cur + 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::history::History;

    #[test]
    pub fn test() {
        let mut h = History::new(1);
        assert_eq!(h.current(), &1);

        h.push(2);
        assert_eq!(h.current(), &2);

        h.push(3);
        assert_eq!(h.current(), &3);

        h.undo();
        assert_eq!(h.current(), &2);

        h.undo();
        assert_eq!(h.current(), &1);

        h.undo();
        assert_eq!(h.current(), &1);

        h.redo();
        assert_eq!(h.current(), &2);

        h.redo();
        assert_eq!(h.current(), &3);

        h.redo();
        assert_eq!(h.current(), &3);
    }
}
