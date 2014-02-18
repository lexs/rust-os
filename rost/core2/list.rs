use core::option::{Option, Some, None};
use core::mem::transmute;
use core::iter::Iterator;
use core::container::Container;

use core2::util;

type Link<T> = Option<~Node<T>>;
pub struct Node<T> {
    value: T,
    next: Link<T>
}

pub struct Rawlink<T> {
    p: *mut T
}

pub struct List<T> {
    head: Link<T>,
    tail: Rawlink<Node<T>>,
    length: uint
}

/// Double-ended DList iterator
//#[deriving(Clone)]
pub struct Items<'a, T> {
    priv head: &'a Link<T>,
    priv length: uint
}

macro_rules! count (
    () => (0);
    ($head:expr $($tail:expr)*) => (
        1 + count!( $($tail)* )
    );
)

macro_rules! link (
    () => (None);
    ($head:expr $($tail:expr)*) => (
        Some(~Node { value: $head, next: link!( $($tail)* ) })
    );
)

macro_rules! list (
    ($($value:expr),*) => (
        List { head: link!( $($value)* ), length: count!( $($value)* ) }
    );
)

impl<T> Node<T> {
    fn new(value: T, next: Link<T>) -> Node<T> {
        Node { value: value, next: next }
    }
}

impl<T> Rawlink<T> {
    fn none() -> Rawlink<T> {
        Rawlink { p: 0 as *mut T }
    }

    fn some(n: &mut T) -> Rawlink<T> {
        Rawlink { p: n }
    }

    fn resolve_immut(&self) -> Option<&T> {
        if self.p == 0 as *mut T {
            None
        } else {
            Some(unsafe { transmute(self.p) })
        }
    }

    fn resolve(&mut self) -> Option<&mut T> {
        if self.p == 0 as *mut T {
            None
        } else {
            Some(unsafe { transmute(self.p) })
        }
    }
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        List { head: None, tail: Rawlink::none(), length: 0 }
    }

    pub fn front<'a>(&'a self) -> Option<&'a T> {
        self.head.as_ref().map(|node| {
            &node.value
        })
    }

    pub fn front_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        self.head.as_mut().map(|node| {
            &mut node.value
        })
    }

    pub fn back<'a>(&'a self) -> Option<&'a T> {
        self.tail.resolve_immut().map(|node| {
            &node.value
        })
    }

    pub fn back_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        self.tail.resolve().map(|node| {
            &mut node.value
        })
    }

    pub fn prepend(&mut self, value: T) {
        let tail = util::replace(&mut self.head, None);
        let mut head = ~Node::new(value, tail);
        match self.tail.resolve() {
            None => self.tail = Rawlink::some(head),
            _ => {}
        }
        self.head = Some(head);
        self.length += 1;
    }

    pub fn append(&mut self, value: T) {
        match self.tail.resolve() {
            None => return self.prepend(value),
            Some(tail) => {
                let mut new_tail = ~Node::new(value, None);
                self.tail = Rawlink::some(new_tail);
                tail.next = Some(new_tail);
                self.length += 1;
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(|node| {
            node.value
        })
    }

    fn pop_front_node(&mut self) -> Option<~Node<T>> {
        self.head.take().map(|mut front_node| {
            match front_node.next.take() {
                Some(node) => self.head = Some(node),
                None => self.tail = Rawlink::none()
            }
            self.length -= 1;
            front_node
        })
    }

    /// Provide a forward iterator
    #[inline]
    pub fn iter<'a>(&'a self) -> Items<'a, T> {
        Items { length: self.len(), head: &self.head }
    }
}

impl<T> Container for List<T> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn len(&self) -> uint {
        self.length
    }
}

impl<'a, A> Iterator<&'a A> for Items<'a, A> {
    #[inline]
    fn next(&mut self) -> Option<&'a A> {
        self.head.as_ref().map(|head| {
            self.head = &head.next;
            self.length -= 1;
            &head.value
        })
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.length, Some(self.length))
    }
}
