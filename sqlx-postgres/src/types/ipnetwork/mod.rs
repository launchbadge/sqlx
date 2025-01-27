// Prefer `ipnet` over `ipnetwork`, as it is more featureful and more widely used.
#[cfg(not(feature = "ipnet"))]
mod ipaddr;

// Parent module is named after the `ipnetwork` crate, this is named after the `IpNetwork` type.
#[allow(clippy::module_inception)]
mod ipnetwork;
