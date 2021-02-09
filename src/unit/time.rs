use std::{env, ffi::CString, mem};

/// first thing I tried with chrono was to use the `%Z` option documented here
/// https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html to print out the local
/// timezone. But as noted in the footnotes https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html#fn7
/// and per https://github.com/chronotope/chrono/issues/288 it was never implemented.
///
/// Simple C example shows this problem has been solved in the C world for ages:
///
/// ```ignore
/// #include <time.h>
/// #include <stdio.h>
/// int main(void)
/// {
///     char outstr[100];
///     time_t t = time(NULL);
///     struct tm result;
///     struct tm *tm = localtime_r(&t, &result);
///     strftime(outstr, sizeof(outstr), "%a %e %b %Y %T %p %Z", tm);
///     printf("LOCAL TIME: %s\n", outstr);
/// }
/// ```
/// $ gcc -Wall time.c -o time
/// $ ./time
/// LOCAL TIME: Mon  8 Feb 2021 09:35:24 AM MST
///
/// `libc` is already a dependency so adding in some code to use `strftime` directly.

/// `Local` provides access to time functions using local time.
pub struct Local(libc::tm);

impl Local {
    /// Create a new local date time from now
    ///
    /// ### Examples
    /// ```ignore
    /// use fungus::prelude::*;
    ///
    /// let time = time::Local::now();
    /// assert_eq!(time.format("%a %e %b %Y %r %Z"), "Sat 12 Dec 2020 8:08:15 PM MST");
    /// ```
    pub fn now() -> Self {
        unsafe {
            let secs = c::time(std::ptr::null());
            let mut t: libc::tm = mem::zeroed();
            c::localtime_r(&secs, &mut t);
            Self(t)
        }
    }

    /// Create a new local date time from a unix timestamp
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// time::set_timezone("America/Denver");
    /// let time = time::Local::timestamp(1_607_789_295);
    /// assert_eq!(time.format("%a %e %b %Y %r %Z"), "Sat 12 Dec 2020 09:08:15 AM MST");
    /// ```
    pub fn timestamp(secs: i64) -> Self {
        unsafe {
            let mut t: libc::tm = mem::zeroed();
            c::localtime_r(&secs, &mut t);
            Self(t)
        }
    }

    /// Format the time using the `fmt` paramaters given which follow the C `strftime` style
    /// formatting documented here https://man.archlinux.org/man/strftime.3
    pub fn format(&self, fmt: &str) -> String {
        strftime(fmt, &self.0)
    }
}

/// `Utc` provides access to time functions using UTC time.
pub struct Utc(libc::tm);

impl Utc {
    /// Create a new Utc date time from now
    ///
    /// ### Examples
    /// ```ignore
    /// use fungus::prelude::*;
    ///
    /// let time = time::Utc::now();
    /// assert_eq!(time.format("%a %e %b %Y %r %Z"), "Sat 12 Dec 2020 04:08:15 PM UTC");
    /// ```
    pub fn now() -> Self {
        unsafe {
            let secs = c::time(std::ptr::null());
            let mut t: libc::tm = mem::zeroed();
            c::gmtime_r(&secs, &mut t);
            Self(t)
        }
    }

    /// Create a new Utc date time from a unix timestamp
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// let time = time::Utc::timestamp(1_607_789_295);
    /// assert_eq!(time.format("%a %e %b %Y %r %Z"), "Sat 12 Dec 2020 04:08:15 PM UTC");
    /// ```
    pub fn timestamp(secs: i64) -> Self {
        unsafe {
            let mut t: libc::tm = mem::zeroed();
            c::gmtime_r(&secs, &mut t);
            Self(t)
        }
    }

    /// Format the time using the `fmt` paramaters given which follow the C `strftime` style
    /// formatting documented here https://man.archlinux.org/man/strftime.3
    pub fn format(&self, fmt: &str) -> String {
        strftime(fmt, &self.0).replace("GMT", "UTC")
    }
}

// Format the time using the `fmt` paramaters given which follow the C `strftime` style
// formatting documented here https://man.archlinux.org/man/strftime.3
pub(crate) fn strftime(fmt: &str, tm: &libc::tm) -> String {
    let buf = [0_u8; 100];
    let f = CString::new(fmt).unwrap(); // time format syntax is limited so risk here is low
    let i: usize = unsafe { c::strftime(buf.as_ptr() as _, buf.len(), f.as_ptr() as *const _, tm) };
    String::from_utf8_lossy(&buf[..i]).to_string()
}

/// Set the timezone from the given value
pub fn set_timezone(tz: &str) {
    env::set_var("TZ", tz);
    unsafe {
        c::tzset();
    }
}

// libc types specific to `time` not exposed by base libc crate
mod c {
    extern "C" {
        // `gmtime_r` converts the unix `timestamp` into a broken-down format and stores it in
        // `result`. `gmtime_r` is the thread safe version and is more than 4x faster than
        // `gmtime`. Typically you get an unix timestamp in UTC from calling `time()`.
        //
        // struct tm* gmtime_r(const time_t* timestamp, struct tm* result);
        // https://man.archlinux.org/man/localtime.3p.html
        //
        // `timestamp` is a pointer to a `time_t` object that contains the time you want to convert
        // `result` is a pointer to a `tm` struct where the function can store the converted time
        pub(crate) fn gmtime_r(timestamp: *const libc::time_t, result: *mut libc::tm);

        // `localtime_r` converts the unix `timestamp` into a broken-down format and stores it in
        // `result`. localtime_r is the thread safe version and is more than 4x faster than
        // `localtime`. Typically you get an unix timestamp in UTC from calling `time()`.
        //
        // struct tm* localtime_r(const time_t* timestamp, struct tm* result);
        // https://man.archlinux.org/man/localtime.3p.html
        //
        // `timestamp` is a pointer to a `time_t` object that contains the time you want to convert
        // `result` is a pointer to a `tm` struct where the function can store the converted time
        pub(crate) fn localtime_r(timestamp: *const libc::time_t, result: *mut libc::tm);

        // `strftime` formats the broken-down time `tm` according to the `fmt` specification to
        // produce a human readable string.
        //
        // `result`. localtime_r is the thread safe version and is more than 4x faster than localtime.
        // Typically you get an unix timestamp in UTC from calling `time()`.
        //
        // size_t strftime(char* s, size_t max, const char* fmt, const struct tm* tm);
        // https://man.archlinux.org/man/strftime.3
        // http://www.qnx.com/developers/docs/qnxcar2/index.jsp?topic=%2Fcom.qnx.doc.neutrino.lib_ref%2Ftopic%2Fl%2Flocaltime_r.html
        //
        // `buf` is a pointer to a buffer where the function can store the formatted time
        // `max` is the max size of the given buffer
        // `fmt` is the format to translate the time into
        // `tm` is the pointer to the time structure were translating
        // returns the length of the buffer used for data
        pub(crate) fn strftime(buf: *mut libc::c_char, max: libc::size_t, fmt: *const libc::c_char, tm: *const libc::tm) -> usize;

        /// Get time in seconds since the the unix Epock i.e. timestamp.
        ///
        /// https://man.archlinux.org/man/time.2.en
        /// `tm` if not NULL will also be updated to the same value as the return
        /// returns the time value
        pub(crate) fn time(tm: *const libc::time_t) -> libc::time_t;

        /// Initialize the time zone name variable from the `TZ` environment variable.
        /// If the `TZ` environment variable is not set the system time zone will be used.
        ///
        /// https://man.archlinux.org/man/core/man-pages/tzset.3.en
        pub(crate) fn tzset();
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_now() {
        println!("{}", Utc::now().format("%a %e %b %Y %r %Z"));
    }

    #[test]
    fn local_now() {
        println!("{}", Local::now().format("%a %e %b %Y %r %Z"));
    }

    #[test]
    fn local_format() {
        set_timezone("America/Denver");
        let fmt = Local::timestamp(1_607_789_295).format("%a %e %b %Y %r %Z");
        assert_eq!(fmt, "Sat 12 Dec 2020 09:08:15 AM MST");
    }

    #[test]
    fn utc_format() {
        let fmt = Utc::timestamp(1_607_789_295).format("%a %e %b %Y %r %Z");
        assert_eq!(fmt, "Sat 12 Dec 2020 04:08:15 PM UTC");
    }
}
