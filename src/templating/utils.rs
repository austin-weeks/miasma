use rand::seq::IndexedRandom;

/// Select a `n` elements randomly from `list`, where `n` is a random number in `range`.
///
/// # Panics
///
/// Panics if `list` is empty.
pub fn select_random_range<T, R>(list: &[T], range: R) -> Vec<T>
where
    T: PartialEq + Clone,
    R: rand::distr::uniform::SampleRange<usize>,
{
    const MAX_TRIES: u8 = 50;
    let mut tries = 0;

    let count = rand::random_range(range);
    let mut out = Vec::with_capacity(count);
    while out.len() < out.capacity() {
        if tries >= MAX_TRIES {
            break;
        }
        tries += 1;

        let pick = list.choose(&mut rand::rng()).expect("");
        if !out.contains(pick) {
            out.push(pick.clone());
        }
    }
    out
}

/// Choose a random element from the list.
///
/// # Panics
///
/// Panics if `list` is empty.
pub fn select_random<T>(list: &[T]) -> &T {
    list.choose(&mut rand::rng())
        .expect("list should not be empty")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn panics_on_empty_list() {
        let list: &[usize] = &[];
        select_random_range(list, 1..=1);
    }

    #[test]
    fn returns_unique_elements() {
        let list = ["1", "2", "1", "1"];
        let result = select_random_range(&list, 2..=2);
        assert_eq!(result.len(), 2);
        assert_ne!(result[0], result[1]);
    }

    #[test]
    fn no_infinite_loop_with_all_equal_elements() {
        let list = ["a", "a", "a", "a"];
        select_random_range(&list, 0..10);
    }
}
