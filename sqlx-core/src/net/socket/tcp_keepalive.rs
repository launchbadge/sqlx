use std::time::Duration;

/// Configures a socket's TCP keepalive parameters.
#[derive(Debug, Clone, Copy)]
pub struct TcpKeepalive {
    #[cfg_attr(
        any(target_os = "openbsd", target_os = "haiku", target_os = "vita"),
        allow(dead_code)
    )]
    time: Option<Duration>,
    #[cfg(not(any(
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
        target_os = "nto",
        target_os = "espidf",
        target_os = "vita",
        target_os = "haiku",
    )))]
    interval: Option<Duration>,
    #[cfg(not(any(
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
        target_os = "windows",
        target_os = "nto",
        target_os = "espidf",
        target_os = "vita",
        target_os = "haiku",
    )))]
    retries: Option<u32>,
}

impl TcpKeepalive {
    /// Returns a new, empty set of TCP keepalive parameters.
    /// The unset parameters will use OS-defined defaults.
    pub const fn new() -> TcpKeepalive {
        TcpKeepalive {
            time: None,
            #[cfg(not(any(
                target_os = "openbsd",
                target_os = "redox",
                target_os = "solaris",
                target_os = "nto",
                target_os = "espidf",
                target_os = "vita",
                target_os = "haiku",
            )))]
            interval: None,
            #[cfg(not(any(
                target_os = "openbsd",
                target_os = "redox",
                target_os = "solaris",
                target_os = "windows",
                target_os = "nto",
                target_os = "espidf",
                target_os = "vita",
                target_os = "haiku",
            )))]
            retries: None,
        }
    }

    /// Set the amount of time after which TCP keepalive probes will be sent on
    /// idle connections.
    ///
    /// This will set `TCP_KEEPALIVE` on macOS and iOS, and
    /// `TCP_KEEPIDLE` on all other Unix operating systems, except
    /// OpenBSD and Haiku which don't support any way to set this
    /// option. On Windows, this sets the value of the `tcp_keepalive`
    /// struct's `keepalivetime` field.
    ///
    /// Some platforms specify this value in seconds, so sub-second
    /// specifications may be omitted.
    pub const fn with_time(self, time: Duration) -> Self {
        Self {
            time: Some(time),
            ..self
        }
    }

    /// Set the value of the `TCP_KEEPINTVL` option. On Windows, this sets the
    /// value of the `tcp_keepalive` struct's `keepaliveinterval` field.
    ///
    /// Sets the time interval between TCP keepalive probes.
    ///
    /// Some platforms specify this value in seconds, so sub-second
    /// specifications may be omitted.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "tvos",
        target_os = "watchos",
        target_os = "windows",
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "tvos",
            target_os = "watchos",
            target_os = "windows",
        )))
    )]
    pub const fn with_interval(self, interval: Duration) -> Self {
        Self {
            interval: Some(interval),
            ..self
        }
    }

    /// Set the value of the `TCP_KEEPCNT` option.
    ///
    /// Set the maximum number of TCP keepalive probes that will be sent before
    /// dropping a connection, if TCP keepalive is enabled on this socket.
    ///
    /// This setter has no effect on Windows.
    #[cfg(all(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "tvos",
        target_os = "watchos",
    )))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "tvos",
            target_os = "watchos",
        ))))
    )]
    pub const fn with_retries(self, retries: u32) -> Self {
        Self {
            retries: Some(retries),
            ..self
        }
    }

    /// Convert `TcpKeepalive` to `socket2::TcpKeepalive`.
    #[doc(hidden)]
    pub(super) const fn socket2(self) -> socket2::TcpKeepalive {
        let mut ka = socket2::TcpKeepalive::new();
        if let Some(time) = self.time {
            ka = ka.with_time(time);
        }
        #[cfg(any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "tvos",
            target_os = "watchos",
            target_os = "windows",
        ))]
        #[cfg_attr(
            docsrs,
            doc(cfg(any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "tvos",
                target_os = "watchos",
                target_os = "windows",
            )))
        )]
        if let Some(interval) = self.interval {
            ka = ka.with_interval(interval);
        }
        #[cfg(all(any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "tvos",
            target_os = "watchos",
        )))]
        #[cfg_attr(
            docsrs,
            doc(cfg(all(any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "tvos",
                target_os = "watchos",
            ))))
        )]
        if let Some(retries) = self.retries {
            ka = ka.with_retries(retries);
        }
        ka
    }
}
