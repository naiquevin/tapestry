use crate::error::Error;
use crate::metadata::Metadata;
use std::path::Path;

pub fn validate() -> Result<(), Error> {
    let path = Path::new("tapestry.toml");
    let metadata = Metadata::try_from(path)?;
    let mistakes = metadata.validate();
    if mistakes.is_empty() {
        println!("All Ok: Manifest file '{}' is valid", path.display());
        Ok(())
    } else {
        println!("Invalid manifest file: '{}'", path.display());
        for mistake in mistakes {
            println!("{}", mistake.err_msg())
        }
        Err(Error::InvalidManifest)
    }
}
