use core::option::{Option, Some, None};
use core::iter::Iterator;
use core::container::Container;

use core2::util;

type Link<T> = Option<~Node<T>>;
struct Node<T> {
    value: T,
    next: Link<T>
}

pub struct List<T> {
    head: Link<T>,
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

impl<T> List<T> {
    pub fn new() -> List<T> {
        List { head: None, length: 0 }
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

    pub fn add(&mut self, value: T) {
        let tail = util::replace(&mut self.head, None);
        self.head = Some(~Node::new(value, tail));
        self.length += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(|node| {
            node.value
        })
    }

    fn pop_front_node(&mut self) -> Option<~Node<T>> {
        self.head.take().map(|mut front_node| {
            self.head = front_node.next.take();
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
