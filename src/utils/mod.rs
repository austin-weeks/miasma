pub mod buffered;
pub mod cow_helpers;
pub mod html_escaper;
pub mod stream_size;

#[macro_export]
/// Ternary expression - equivalent to `if cond { a } else { b }`.
///
/// ```
/// use miasma::ternary;
///
/// let condition = true;
/// let result = ternary!(condition, "true case", "false case");
/// assert_eq!(result, "true case");
/// ```
macro_rules! ternary {
    ($condition:expr, $true_case:expr, $false_case:expr$(,)?) => {
        if $condition { $true_case } else { $false_case }
    };
}
