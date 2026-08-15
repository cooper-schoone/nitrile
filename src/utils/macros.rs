/// Wraps a block or closure so that errors caught with ? do not propagate to the outer scope.
///
/// # Example
///
/// ```
/// # fn might_fail() -> Result<()> {Ok(())}
/// # fn consume_string(_s: String) -> Result<()> {Ok(())}
/// # #[macro_use] extern crate nitrile;
/// use color_eyre::Result;
///
/// # fn main() {
/// let block_result: Result<i32> = attempt!({
///     might_fail()?;
///     Ok(0)
/// });
/// let owned = String::from("test");
/// let closure_result: Result<i32> = attempt!(|| {
///     consume_string(owned)?;
///     Ok(1)
/// });
/// assert_eq!(block_result.unwrap(), 0);
/// assert_eq!(closure_result.unwrap(), 1);
/// # }
/// ```
#[macro_export]
macro_rules! attempt {
    ($body:block) => {
        (|| $body)()
    };
    ($closure:expr $(,)?) => {
        ($closure)()
    };
}

/**
Displays a spinner with a specified message that automatically clears after a provided code
block or closure returns. This allows spinners to safely be cleared if the enclosed block or
closure returns an error.

# Example

```rust
# fn perform_action() -> Result<()> {Ok(())}
# fn consume_string(_s: String) -> Result<()> {Ok(())}
# #[macro_use] extern crate nitrile;
use color_eyre::Result;

# fn main() {
let block_result: Result<()> = spinner!("Running...", {
    perform_action()?;
    Ok(())
});
let owned = String::from("test");
let closure_result: Result<()> = spinner!("Running...", || {
    consume_string(owned)?;
    Ok(())
});
assert!(block_result.is_ok());
assert!(closure_result.is_ok());
# }
```
*/
#[macro_export]
macro_rules! spinner {
    ($msg:expr, $body:block) => {{
        use indicatif::ProgressBar;
        use std::time::Duration;

        let spinner = ProgressBar::new_spinner().with_message($msg);
        spinner.enable_steady_tick(Duration::from_millis(100));
        let result = $crate::attempt!($body);
        spinner.finish_and_clear();
        result
    }};
    ($msg:expr, $closure:expr $(,)?) => {{
        use indicatif::ProgressBar;
        use std::time::Duration;

        let spinner = ProgressBar::new_spinner().with_message($msg);
        spinner.enable_steady_tick(Duration::from_millis(100));
        let result = ($closure)();
        spinner.finish_and_clear();
        result
    }};
}
