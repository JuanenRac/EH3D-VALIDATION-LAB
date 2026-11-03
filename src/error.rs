// SPDX-License-Identifier: GPL-3.0-or-later
//! The typed errors of the laboratory. Nothing in the library panics on bad input: every refusal is one of these.

use std::path::PathBuf;

/// Everything that can go wrong while reading, checking or comparing.
#[derive(Debug, thiserror::Error)]
pub enum LabError {
    /// The configuration file has a line that is not `key = value`.
    #[error("configuration line {line}: expected `key = value`")]
    ConfigSyntax {
        /// 1-based line number.
        line: usize,
    },
    /// The configuration file names a key this version does not know.
    #[error("configuration line {line}: unknown key `{key}`")]
    ConfigUnknownKey {
        /// 1-based line number.
        line: usize,
        /// The key as written.
        key: String,
    },
    /// The configuration value cannot be used.
    #[error("configuration line {line}: bad value for `{key}`")]
    ConfigBadValue {
        /// 1-based line number.
        line: usize,
        /// The key as written.
        key: String,
    },
    /// A path inside a manifest is not a safe relative path.
    #[error("manifest line {line}: unsafe or malformed path")]
    BadPath {
        /// 1-based line number (0 when the path did not come from a line).
        line: usize,
    },
    /// A manifest line is not `<64 hex digits>  <path>`.
    #[error("manifest line {line}: expected a lower-case SHA-256 and a path")]
    ManifestSyntax {
        /// 1-based line number.
        line: usize,
    },
    /// Two manifest paths are the same, or differ only in letter case.
    #[error("manifest line {line}: duplicate path")]
    DuplicatePath {
        /// 1-based line number (0 when the path did not come from a line).
        line: usize,
    },
    /// A bound of the configuration was exceeded.
    #[error("limit exceeded: {what}")]
    TooLarge {
        /// Which bound.
        what: &'static str,
    },
    /// A directory holds something that is not a plain file or folder, or a name that is not UTF-8.
    #[error("unsupported entry in the golden set (link, device or non-UTF-8 name)")]
    UnsupportedEntry,
    /// The same copy was named twice, so it cannot count as redundant.
    #[error("the same copy was given more than once")]
    DuplicateCopy,
    /// A results line is not `case<TAB>level<TAB>outcome`.
    #[error("results line {line}: expected `case<TAB>level<TAB>pass|fail:<class>`")]
    ResultsSyntax {
        /// 1-based line number.
        line: usize,
    },
    /// A failure was reported without one of the three classes.
    #[error("results line {line}: a failure must carry a class")]
    UnclassifiedFailure {
        /// 1-based line number.
        line: usize,
    },
    /// The same case and level appear twice in one results file.
    #[error("results line {line}: the case and level are already reported")]
    DuplicateResult {
        /// 1-based line number.
        line: usize,
    },
    /// A line of a golden or observed file is not `case<TAB>L<n>:name<TAB>value`, or has a fourth column where none is allowed.
    #[error("oracle line {line}: expected `case<TAB>L<n>:check<TAB>value[<TAB>class]`")]
    OracleSyntax {
        /// 1-based line number.
        line: usize,
    },
    /// A working copy was asked for in a place that is not safe: inside the original folder, around it, or over files that are already there.
    #[error("the target of the copy is inside the original, around it, or not empty")]
    UnsafeTarget,
    /// A file read back from the copy is not the file that was read from the original.
    #[error("a copied file does not read back as the original")]
    CopyMismatch,
    /// The baseline file is not a baseline written by this tool.
    #[error("the baseline cannot be read")]
    BadBaseline,
    /// A file could not be read.
    #[error("cannot read {path}: {source}")]
    Io {
        /// The path that failed (as given by the caller).
        path: PathBuf,
        /// The operating system error.
        #[source]
        source: std::io::Error,
    },
}
