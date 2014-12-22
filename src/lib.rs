extern crate libc;
extern crate "magic-sys" as ffi;
extern crate regex;

use libc::size_t;
use std::path::Path;
use std::ptr;
use std::error;

/// Bitmask flags which control `libmagic` behaviour
#[unstable]
pub mod flags {
    use libc::c_int;

    bitflags! {
        #[doc = "Bitmask flags that specify how `Cookie` functions should behave"]
        flags CookieFlags: c_int {
            #[doc = "No flags"]
            const NONE              = 0x000000,

            #[doc = "Turn on debugging"]
            const DEBUG             = 0x000001,

            #[doc = "Follow symlinks"]
            const SYMLINK           = 0x000002,

            #[doc = "Check inside compressed files"]
            const COMPRESS          = 0x000004,

            #[doc = "Look at the contents of devices"]
            const DEVICES           = 0x000008,

            #[doc = "Return the MIME type"]
            const MIME_TYPE         = 0x000010,

            #[doc = "Return all matches"]
            const CONTINUE          = 0x000020,

            #[doc = "Print warnings to stderr"]
            const CHECK             = 0x000040,

            #[doc = "Restore access time on exit"]
            const PRESERVE_ATIME    = 0x000080,

            #[doc = "Don't translate unprintable chars"]
            const RAW               = 0x000100,

            #[doc = "Handle `ENOENT` etc as real errors"]
            const ERROR             = 0x000200,

            #[doc = "Return the MIME encoding"]
            const MIME_ENCODING     = 0x000400,

            #[doc = "Return the MIME type and encoding"]
            const MIME              = MIME_TYPE.bits
                                     | MIME_ENCODING.bits,

            #[doc = "Return the Apple creator and type"]
            const APPLE             = 0x000800,

            #[doc = "Don't check for compressed files"]
            const NO_CHECK_COMPRESS = 0x001000,

            #[doc = "Don't check for tar files"]
            const NO_CHECK_TAR      = 0x002000,

            #[doc = "Don't check magic entries"]
            const NO_CHECK_SOFT     = 0x004000,

            #[doc = "Don't check application type"]
            const NO_CHECK_APPTYPE  = 0x008000,

            #[doc = "Don't check for elf details"]
            const NO_CHECK_ELF      = 0x010000,

            #[doc = "Don't check for text files"]
            const NO_CHECK_TEXT     = 0x020000,

            #[doc = "Don't check for cdf files"]
            const NO_CHECK_CDF      = 0x040000,

            #[doc = "Don't check tokens"]
            const NO_CHECK_TOKENS   = 0x100000,

            #[doc = "Don't check text encodings"]
            const NO_CHECK_ENCODING = 0x200000,

            #[doc = "No built-in tests; only consult the magic file"]
            const NO_CHECK_BUILTIN  = NO_CHECK_COMPRESS.bits
                                     | NO_CHECK_TAR.bits
                                     | NO_CHECK_APPTYPE.bits
                                     | NO_CHECK_ELF.bits
                                     | NO_CHECK_TEXT.bits
                                     | NO_CHECK_CDF.bits
                                     | NO_CHECK_TOKENS.bits
                                     | NO_CHECK_ENCODING.bits
                                     | 0,
	    }
    }
}


/// Returns the version of this crate in the format `MAJOR.MINOR.PATCH`.
#[unstable]
pub fn version() -> String {
    // TODO: There's also an optional _PRE part
    let (maj, min, pat) = (
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
    );

    format!("{}.{}.{}", maj, min, pat)
}


/// Represents a magic error.
/// For the most part you should be using the `Error` trait
/// to interact with rather than this struct.
#[experimental]
#[deriving(PartialEq, Eq, Clone, Show)]
pub struct MagicError {
    pub desc: String,
}

impl error::Error for MagicError {
    fn description(&self) -> &str {
        self.desc.as_slice()
    }
}


#[unstable]
pub struct Cookie {
    cookie: *const self::ffi::Magic,
}

#[unstable]
impl Drop for Cookie {
    fn drop(&mut self) { unsafe { self::ffi::magic_close(self.cookie) } }
}

#[experimental]
impl Cookie {
    fn last_error(&self) -> Option<MagicError> {
        let cookie = self.cookie;

        unsafe {
            let e = self::ffi::magic_error(cookie);
            if e.is_null() {
                None
            } else {
                Some(self::MagicError{desc: String::from_raw_buf(e as *const u8),})
            }
        }
    }

    fn magic_failure(&self) -> MagicError {
        match self.last_error() {
            Some(e) => e,
            None => self::MagicError{desc: "unknown error".to_string(),}
        }
    }

    pub fn file(&self, filename: &Path) -> Result<String, MagicError> {
        unsafe {
            let cookie = self.cookie;
            let s = filename.with_c_str(|filename| self::ffi::magic_file(cookie, filename));
            if s.is_null() { Err(self.magic_failure()) } else { Ok(String::from_raw_buf(s as *const u8)) }
        }
    }

    pub fn buffer(&self, buffer: &[u8]) -> Result<String, MagicError> {
        unsafe {
            let buffer_len = buffer.len() as size_t;
            let pbuffer = buffer.as_ptr();
            let s = self::ffi::magic_buffer(self.cookie, pbuffer, buffer_len);
            if s.is_null() { Err(self.magic_failure()) } else { Ok(String::from_raw_buf(s as *const u8)) }
        }
    }

    pub fn error(&self) -> Option<String> {
        unsafe {
            let s = self::ffi::magic_error(self.cookie);
            if s.is_null() { None } else { Some(String::from_raw_buf(s as *const u8)) }
        }
    }

    pub fn set_flags(&self, flags: self::flags::CookieFlags) -> bool {
        unsafe {
            self::ffi::magic_setflags(self.cookie, flags.bits()) != -1
        }
    }

    // TODO: check, compile, list and load mostly do the same, refactor!
    // TODO: ^ also needs to implement multiple databases, possibly waiting for the Path reform

    pub fn check(&self, filenames: &[Path]) -> Result<(), MagicError> {
        unsafe {
            let cookie = self.cookie;

            let ret = match filenames.len() {
                0 => self::ffi::magic_load(cookie, ptr::null()),
                1 => filenames[0].with_c_str(|filename| self::ffi::magic_check(cookie, filename)),
                _ => unimplemented!(),
            };

            if 0 == ret { Ok(()) } else { Err(self.magic_failure()) }
        }
    }

    pub fn compile(&self, filenames: &[Path]) -> Result<(), MagicError> {
        unsafe {
            let cookie = self.cookie;

            let ret = match filenames.len() {
                0 => self::ffi::magic_load(cookie, ptr::null()),
                1 => filenames[0].with_c_str(|filename| self::ffi::magic_compile(cookie, filename)),
                _ => unimplemented!(),
            };

            if 0 == ret { Ok(()) } else { Err(self.magic_failure()) }
        }
    }

    pub fn list(&self, filenames: &[Path]) -> Result<(), MagicError> {
        unsafe {
            let cookie = self.cookie;

            let ret = match filenames.len() {
                0 => self::ffi::magic_load(cookie, ptr::null()),
                1 => filenames[0].with_c_str(|filename| self::ffi::magic_list(cookie, filename)),
                _ => unimplemented!(),
            };

            if 0 == ret { Ok(()) } else { Err(self.magic_failure()) }
        }
    }

    pub fn load(&self, filenames: &[Path]) -> Result<(), MagicError> {
        unsafe {
            let cookie = self.cookie;

            let ret = match filenames.len() {
                0 => self::ffi::magic_load(cookie, ptr::null()),
                1 => filenames[0].with_c_str(|filename| self::ffi::magic_load(cookie, filename)),
                _ => unimplemented!(),
            };

            if 0 == ret { Ok(()) } else { Err(self.magic_failure()) }
        }
    }

    pub fn open(flags: self::flags::CookieFlags) -> Result<Cookie, MagicError> {
        unsafe {
            let cookie = self::ffi::magic_open((flags | self::flags::ERROR).bits());
            if cookie.is_null() { Err(self::MagicError{desc: "errno".to_string(),}) } else { Ok(Cookie {cookie: cookie,}) }
        }
    }
}

#[cfg(test)]
mod tests {
    use regex;

    use super::Cookie;
	use super::flags;

    // Using relative paths to test files should be fine, since cargo doc
    // http://doc.crates.io/build-script.html#inputs-to-the-build-script
    // states that cwd == CARGO_MANIFEST_DIR

    #[test]
    fn file() {
        let cookie = Cookie::open(flags::NONE).ok().unwrap();
        assert!(cookie.load(vec![Path::new("data/tests/db-images-png")].as_slice()).is_ok());

        let path = Path::new("data/tests/rust-logo-128x128-blk.png");

        assert_eq!(cookie.file(&path).ok().unwrap().as_slice(), "PNG image data, 128 x 128, 8-bit/color RGBA, non-interlaced");

        cookie.set_flags(flags::MIME_TYPE);
        assert_eq!(cookie.file(&path).ok().unwrap().as_slice(), "image/png");

        cookie.set_flags(flags::MIME_TYPE | flags::MIME_ENCODING);
        assert_eq!(cookie.file(&path).ok().unwrap().as_slice(), "image/png; charset=binary");
    }

    #[test]
    fn buffer() {
        let cookie = Cookie::open(flags::NONE).ok().unwrap();
        assert!(cookie.load(vec![Path::new("data/tests/db-python")].as_slice()).is_ok());

        let s = b"#!/usr/bin/env python\nprint('Hello, world!')";
        assert_eq!(cookie.buffer(s).ok().unwrap().as_slice(), "Python script, ASCII text executable");

        cookie.set_flags(flags::MIME_TYPE);
        assert_eq!(cookie.buffer(s).ok().unwrap().as_slice(), "text/x-python");
    }

    #[test]
    fn file_error() {
        let cookie = Cookie::open(flags::NONE | flags::ERROR).ok().unwrap();
        assert!(cookie.load(vec![].as_slice()).is_ok());

        let ret = cookie.file(&Path::new("non-existent_file.txt"));
        assert!(ret.is_err());
        assert_eq!(ret.err().unwrap().desc.as_slice(), "cannot stat `non-existent_file.txt' (No such file or directory)");
    }

    #[test]
    fn load_default() {
        let cookie = Cookie::open(flags::NONE | flags::ERROR).ok().unwrap();
        assert!(cookie.load(vec![].as_slice()).is_ok());
    }

    #[test]
    fn load_one() {
        let cookie = Cookie::open(flags::NONE | flags::ERROR).ok().unwrap();
        assert!(cookie.load(vec![
                                    Path::new("data/tests/db-images-png")
                                ].as_slice()).is_ok());
    }

    #[test]
    fn load_multiple() {
        let cookie = Cookie::open(flags::NONE | flags::ERROR).ok().unwrap();
        assert!(cookie.load(vec![
                                Path::new("data/tests/db-images-png"),
                                Path::new("data/tests/db-python"),
                            ].as_slice()).is_ok());
    }

    #[test]
    fn version() {
        assert!(regex::is_match(r"\d+\.\d+.\d+", super::version().as_slice()).ok().unwrap());
    }
}
