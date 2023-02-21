#[cfg(not(any(feature = "cli")))]
fn main() {}

#[cfg(feature = "cli")]
fn main() -> fasdasm::prelude::FdResult<()> {
    fasdasm::cli::init(&fasdasm::prelude::CFG)
}

#[cfg(test)]
mod test {}
