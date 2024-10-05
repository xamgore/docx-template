#[macro_export]

/// Macro which takes a string literal and returns an expression that evaluates to a `&'static Regex`.
///
/// Can be useful to avoid the “compile regex on every loop iteration” problem.
macro_rules! regex {
  ($re:literal $(,)?) => {{
    static RE: std::sync::OnceLock<::regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| ::regex::Regex::new($re).unwrap())
  }};
}
