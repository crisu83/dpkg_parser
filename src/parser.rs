use super::ast::*;
use std::{error, fmt, io::Write, str::from_utf8};

#[derive(Debug, Clone)]
enum FieldName {
    Package,
    // Status,
    // MultiArch
    // Priority,
    // Section,
    // InstalledSize,
    // Maintainer,
    // Architecture,
    // Source,
    // Version,
    // Replaces,
    // Provides,
    Depends,
    // Suggests,
    // Conflicts,
    Description,
    // OriginalMaintainer,
    // Homepage,
    // PythonVersion,
}

impl ToString for FieldName {
    fn to_string(&self) -> String {
        match self {
            FieldName::Package => String::from("Package"),
            // FieldName::Status => String::from("Status"),
            // FieldName::MultiArch => String::from("Multi-Arch"),
            // FieldName::Priority => String::from("Priority"),
            // FieldName::Section => String::from("Section"),
            // FieldName::InstalledSize => String::from("Installed-Size"),
            // FieldName::Maintainer => String::from("Maintainer"),
            // FieldName::Architecture => String::from("Architecture"),
            // FieldName::Source => String::from("Source"),
            // FieldName::Version => String::from("Version"),
            // FieldName::Replaces => String::from("Replaces"),
            // FieldName::Provides => String::from("Provides"),
            FieldName::Depends => String::from("Depends"),
            // FieldName::Suggests => String::from("Suggests"),
            // FieldName::Conflicts => String::from("Conflicts"),
            FieldName::Description => String::from("Description"),
            // FieldName::OriginalMaintainer => String::from("Original-Maintainer"),
            // FieldName::Homepage => String::from("Homepage"),
            // FieldName::PythonVersion => String::from("Python-Version"),
        }
    }
}

/// A result from a parsing operation.
type ParseResult<T> = Result<T, ParseError>;

/// Describes an error that may occur when parsing a source string.
#[derive(Debug, Clone)]
pub enum ParseError {
    PackageNameNotFound(String),
}

impl ParseError {
    fn write_error(f: &mut fmt::Formatter<'_>, error: &str, source: &str) -> fmt::Result {
        writeln!(f, "{}", error)?;
        writeln!(f)?;
        write!(f, "{}", source)?;
        Ok(())
    }
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::PackageNameNotFound(s) => {
                ParseError::write_error(f, "package name not found", s)
            }
        }
    }
}

/// Parses all packages from a source string.
///
/// # Examples
///
/// ```
/// use dpkg_parser::parser;
///
/// let result = parser::parse("\
/// Package: libssl1.0.0
/// Status: install ok installed
/// Multi-Arch: same
/// Priority: important
/// Section: libs
/// Installed-Size: 2836
/// Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
/// Architecture: amd64
/// Source: openssl
/// Version: 1.0.1-4ubuntu5.5
/// Depends: libc6 (>= 2.14), zlib1g (>= 1:1.1.4), debconf (>= 0.5) | debconf-2.0
/// Pre-Depends: multiarch-support
/// Breaks: openssh-client (<< 1:5.9p1-4), openssh-server (<< 1:5.9p1-4)
/// Description: SSL shared libraries
///     libssl and libcrypto shared libraries needed by programs like
///     apache-ssl, telnet-ssl and openssh.
///     .
///     It is part of the OpenSSL implementation of SSL.
/// Original-Maintainer: Debian OpenSSL Team <pkg-openssl-devel@lists.alioth.debian.org>")
///     .unwrap();
///
/// assert_eq!(result.packages[0].name, "libssl1.0.0");
/// ```
pub fn parse(source: &str) -> ParseResult<Document> {
    let mut packages = Vec::new();
    let mut buf = Vec::new();

    // append an empty line to the end
    writeln!(&mut buf, "{}", source).unwrap();
    writeln!(&mut buf).unwrap();
    let source = from_utf8(&buf[..]).unwrap();

    let mut buf = Vec::new();

    for line in source.lines() {
        if !line.is_empty() {
            writeln!(&mut buf, "{}", line).unwrap();
        } else {
            let s = from_utf8(&buf[..]).unwrap();
            match parse_package(s) {
                Ok(package) => {
                    packages.push(package);
                    buf.clear();
                }
                Err(err) => return Err(err),
            }
        }
    }

    Ok(Document::new(packages))
}

fn parse_package(source: &str) -> ParseResult<Package> {
    let name = parse_field(FieldName::Package, source).unwrap();

    if name.is_empty() {
        return Err(ParseError::PackageNameNotFound(source.to_string()));
    }

    let description = parse_field(FieldName::Description, source).unwrap();
    let depends = parse_field(FieldName::Depends, source).unwrap();
    let depends = parse_libraries(&depends).unwrap();

    Ok(Package::new(name, description, depends))
}

fn parse_field(field_name: FieldName, source: &str) -> ParseResult<String> {
    let mut buf = Vec::new();
    let pat = format!("{}:", field_name.to_string());

    for line in source.lines() {
        let field_name = field_name.to_string();
        let indented = line.chars().next().unwrap() == ' ';

        if line.starts_with(&pat) {
            let replace = format!("{}: ", field_name);
            writeln!(&mut buf, "{}", line.replace(&replace, "")).unwrap();
        } else if !buf.is_empty() {
            if indented {
                writeln!(&mut buf, "{}", line.trim()).unwrap();
            } else {
                break;
            }
        }
    }

    // convert the buffer to a string and strip the last linebreak
    let value = from_utf8(&buf[..]).unwrap().trim().to_string();

    Ok(value)
}

fn parse_libraries(source: &str) -> ParseResult<Vec<Library>> {
    let mut libraries = Vec::new();

    if !source.is_empty() {
        libraries = source
            .split(", ")
            .map(|s| {
                let vec = s.split(" | ").collect::<Vec<&str>>();
                let name = vec[0].to_string();
                let alternates = vec[1..].into_iter().map(|s| s.to_string()).fold(
                    Vec::new(),
                    |mut acc, value| {
                        acc.push(value);
                        acc
                    },
                );

                Library::new(name, alternates)
            })
            .collect();
    }

    Ok(libraries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let source = "\
Package: libws-commons-util-java
Status: install ok installed
Priority: optional
Section: java
Installed-Size: 101
Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
Architecture: all
Version: 1.0.1-7
Description: Common utilities from the Apache Web Services Project
    This is a small collection of utility classes, that allow high
    performance XML processing based on SAX.
Original-Maintainer: Debian Java Maintainers <pkg-java-maintainers@lists.alioth.debian.org>
Homepage: http://ws.apache.org/commons/util/

Package: python-pkg-resources
Status: install ok installed
Priority: optional
Section: python
Installed-Size: 175
Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
Architecture: all
Source: distribute
Version: 0.6.24-1ubuntu1
Replaces: python2.3-setuptools, python2.4-setuptools
Provides: python2.6-setuptools, python2.7-setuptools
Depends: python (>= 2.6), python (<< 2.8)
Suggests: python-distribute, python-distribute-doc
Conflicts: python-setuptools (<< 0.6c8-3), python2.3-setuptools (<< 0.6b2), python2.4-setuptools (<< 0.6b2)
Description: Package Discovery and Resource Access using pkg_resources
    The pkg_resources module provides an API for Python libraries to
    access their resource files, and for extensible applications and
    frameworks to automatically discover plugins.  It also provides
    runtime support for using C extensions that are inside zipfile-format
    eggs, support for merging packages that have separately-distributed
    modules or subpackages, and APIs for managing Python's current
    \"working set\" of active packages.
Original-Maintainer: Matthias Klose <doko@debian.org>
Homepage: http://packages.python.org/distribute
Python-Version: 2.6, 2.7

Package: tcpd
Status: install ok installed
Multi-Arch: foreign
Priority: optional
Section: net
Installed-Size: 132
Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
Architecture: amd64
Source: tcp-wrappers
Version: 7.6.q-21
Replaces: libwrap0 (<< 7.6-8)
Depends: libc6 (>= 2.4), libwrap0 (>= 7.6-4~)
Description: Wietse Venema's TCP wrapper utilities
    Wietse Venema's network logger, also known as TCPD or LOG_TCP.
    .
    These programs log the client host name of incoming telnet,
    ftp, rsh, rlogin, finger etc. requests.
    .
    Security options are:
    - access control per host, domain and/or service;
    - detection of host name spoofing or host address spoofing;
    - booby traps to implement an early-warning system.
Original-Maintainer: Marco d'Itri <md@linux.it>";

        let result = parse(source).unwrap();

        assert_eq!(result.packages.len(), 3);
        assert_eq!(result.packages[0].name, "libws-commons-util-java");
    }

    const PACKAGE: &str = "\
Package: libssl1.0.0
Status: install ok installed
Multi-Arch: same
Priority: important
Section: libs
Installed-Size: 2836
Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
Architecture: amd64
Source: openssl
Version: 1.0.1-4ubuntu5.5
Depends: libc6 (>= 2.14), zlib1g (>= 1:1.1.4), debconf (>= 0.5) | debconf-2.0
Pre-Depends: multiarch-support
Breaks: openssh-client (<< 1:5.9p1-4), openssh-server (<< 1:5.9p1-4)
Description: SSL shared libraries
    libssl and libcrypto shared libraries needed by programs like
    apache-ssl, telnet-ssl and openssh.
    .
    It is part of the OpenSSL implementation of SSL.
Original-Maintainer: Debian OpenSSL Team <pkg-openssl-devel@lists.alioth.debian.org>";

    #[test]
    fn test_parse_name_field() {
        assert_eq!(
            parse_field(FieldName::Package, PACKAGE).unwrap(),
            "libssl1.0.0"
        );
    }

    #[test]
    fn test_parse_depends_field() {
        assert_eq!(
            parse_field(FieldName::Depends, PACKAGE).unwrap(),
            "libc6 (>= 2.14), zlib1g (>= 1:1.1.4), debconf (>= 0.5) | debconf-2.0"
        );
    }

    #[test]
    fn test_parse_description_field() {
        assert_eq!(
            parse_field(FieldName::Description, PACKAGE).unwrap(),
            "SSL shared libraries
libssl and libcrypto shared libraries needed by programs like
apache-ssl, telnet-ssl and openssh.
.
It is part of the OpenSSL implementation of SSL."
        );
    }

    #[test]
    fn test_parse_libraries() {
        let result =
            parse_libraries("libc6 (>= 2.14), zlib1g (>= 1:1.1.4), debconf (>= 0.5) | debconf-2.0")
                .unwrap();

        assert_eq!(result[0].name, "libc6 (>= 2.14)");
        assert!(result[0].alternates.is_empty());
        assert_eq!(result[1].name, "zlib1g (>= 1:1.1.4)");
        assert!(result[1].alternates.is_empty());
        assert_eq!(result[2].name, "debconf (>= 0.5)");
        assert_eq!(result[2].alternates.len(), 1);
        assert_eq!(result[2].alternates[0], "debconf-2.0");
    }
}
