pub mod axes;
mod device;
pub mod evdev;
#[cfg(feature = "host")]
mod host;
#[cfg(feature = "host")]
mod keys;
mod mask;
#[cfg(feature = "host")]
mod pad;
pub mod trace;
#[cfg(feature = "android")]
mod android;

#[cfg(feature = "android")]
pub use android::AndroidInput;
pub use device::DeviceInput;
#[cfg(feature = "host")]
pub use host::HostInput;
pub use mask::Pad;
