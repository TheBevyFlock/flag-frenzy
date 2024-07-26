#[cfg(not(feature = "always-required"))]
compile_error!("`always-required` is not enabled.");

#[cfg(not(any(feature = "choose-required-1", feature = "choose-required-2")))]
compile_error!("Neither `choose-required-1` nor `choose-required-2` are enabled.");

#[cfg(all(feature = "incompatible-1", feature = "incompatible-2"))]
compile_error!("Both `incompatible-1` and `incompatible-2` are enabled.");

#[cfg(feature = "always-fails")]
compile_error!("`always-fails` is enabled.");
