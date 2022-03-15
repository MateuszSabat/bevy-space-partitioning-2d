pub enum Exclusive<T1, T2> {
    First(T1),
    Second(T2),
}

pub enum ExclusiveBox<T1, T2> {
    First(Box<T1>),
    Second(Box<T2>),
}

pub mod iterators {
    pub fn select_many<'a, E1, E2, It1, It2>(iterator: &'a mut It1, select: fn(E1) -> It2) ->  impl Iterator<Item = E2> + 'a where E1: 'a, E2: 'a, It1: Iterator<Item = E1> + 'a, It2: Iterator<Item = E2> + 'a {
        let mut it = SelectManyIterator::<'a, E1, E2, It1, It2> {
            root: iterator,
            current: None,
            select: select,
        };
        it.set_next_current();
        it
    }

    struct SelectManyIterator<'a, E1, E2, It1, It2> where It1: Iterator<Item = E1>, It2: Iterator<Item = E2> {
        root: &'a mut It1,
        current: Option<It2>,
        select: fn(E1) -> It2,
    }

    impl<'a, E1, E2, It1, It2> SelectManyIterator<'a, E1, E2, It1, It2> where It1: Iterator<Item = E1>, It2: Iterator<Item = E2> {
        fn set_next_current(&mut self){
            self.current = match self.root.next() {
                Some(e1) => Some((self.select)(e1)),
                None => None,
            }
        }
    }

    impl<'a, E1, E2, It1, It2> Iterator for SelectManyIterator<'a, E1, E2, It1, It2> where It1: Iterator<Item = E1>, It2: Iterator<Item = E2> {
        type Item = E2;

        fn next(&mut self) -> Option<Self::Item> {
            match &mut self.current {
                Some(current) => match current.next() {
                    Some(e) => Some(e),
                    None => {
                        self.set_next_current();
                        self.next()
                    }
                }
                None => None,
            }
        }
    }

    pub fn flatten<'a, E: 'a>(iterator: &'a mut impl Iterator<Item = impl Iterator<Item = E> + 'a>) -> impl Iterator<Item = E> + 'a {
        select_many(iterator, |nested| nested)
    }
}