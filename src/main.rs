#[cfg(not(any(feature = "cli")))]
fn main() {}

#[cfg(feature = "cli")]
fn main() -> litedasm::prelude::FdResult<()> {
    litedasm::cli::init(&litedasm::prelude::CFG)
}

#[cfg(test)]
mod test {}
