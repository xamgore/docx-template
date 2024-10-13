/// Combine all iterator elements into one `String`, separated by `sep`.
///
/// Use the `Display` implementation of each element.
pub fn join<I: Iterator>(mut iter: I, sep: &str) -> String
where
  I::Item: std::fmt::Display,
{
  use std::fmt::Write;
  match iter.next() {
    None => String::new(),
    Some(first_elt) => {
      // estimate lower bound of capacity needed
      let (lower, _) = iter.size_hint();
      let mut result = String::with_capacity(sep.len() * lower);
      write!(&mut result, "{}", first_elt).unwrap();
      iter.for_each(|elt| {
        result.push_str(sep);
        write!(&mut result, "{}", elt).unwrap();
      });
      result
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_join() {
    assert_eq!(join(["a", "b", "c"].into_iter(), ", "), "a, b, c");
    assert_eq!(join([1, 2, 3].into_iter(), ", "), "1, 2, 3");
  }
}
