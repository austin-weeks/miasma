use rand::seq::IndexedRandom;

/// Select `n` unique elements randomly from `list`, where `n` is a random number in `range`.
///
/// If `list` does not contain enough unique elements to meet the selected count,
/// all unique elements will be returned.
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

        let Some(pick) = list.choose(&mut rand::rng()) else {
            return out;
        };
        if !out.contains(pick) {
            out.push(pick.clone());
            tries = 0;
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
    fn returns_empty_vec_on_empty_list() {
        let list: &[usize] = &[];
        assert!(select_random_range(list, 1..=1).is_empty());
    }

    #[test]
    fn returns_unique_elements() {
        let list = ["1", "2", "1", "1"];
        let result = select_random_range(&list, 2..=2);
        assert_eq!(result.len(), 2);
        assert_ne!(result[0], result[1]);
    }

    #[test]
    fn all_equal_elements() {
        let list = ["a", "a", "a", "a"];
        let result = select_random_range(&list, 2..10);
        assert_eq!(result, vec!["a"]);
    }
}
