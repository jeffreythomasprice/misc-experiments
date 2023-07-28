use std::mem::replace;

#[derive(Debug)]
pub struct List {
    head: Link,
}

#[derive(Debug)]
struct Node {
    value: i32,
    next: Link,
}

#[derive(Debug)]
enum Link {
    Empty,
    More(Box<Node>),
}

impl List {
    pub fn new() -> Self {
        Self { head: Link::Empty }
    }

    pub fn push(&mut self, value: i32) {
        let new_node = Node {
            value,
            next: replace(&mut self.head, Link::Empty),
        };
        self.head = Link::More(new_node.into());
    }

    pub fn pop(&mut self) -> Option<i32> {
        match std::mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                let result = node.value;
                self.head = node.next;
                Some(result)
            }
        }
    }
}

impl Drop for List {
    fn drop(&mut self) {
        let mut hoisted = std::mem::replace(&mut self.head, Link::Empty);
        while let Link::More(mut node) = hoisted {
            hoisted = std::mem::replace(&mut node.next, Link::Empty);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        list.push(4);
        list.push(5);

        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn big_list() {
        let mut list = List::new();
        for i in 0..1_000_000 {
            list.push(i);
        }
    }
}
