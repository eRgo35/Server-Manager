use std::fs;
use std::path::PathBuf;

use sm_services::{HostKeyStore, HostKeyVerdict, ServiceError};

/// File-backed TOFU ("trust on first use") known-hosts store.
///
/// One `host:port SHA256:<base64 fingerprint>` per line; `#` starts a comment.
pub struct FileHostKeyStore {
    path: PathBuf,
}

impl FileHostKeyStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Fingerprint currently stored for `host:port`, if any.
    fn stored_fingerprint(&self, key: &str) -> Option<String> {
        let text = fs::read_to_string(&self.path).ok()?;
        parse_stored(&text, key)
    }
}

/// Reads the fingerprint stored for `key` in known-hosts file text.
///
/// The first matching line wins.
fn parse_stored(text: &str, key: &str) -> Option<String> {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .find_map(|line| {
            let (host, fp) = line.split_once(' ')?;
            (host.trim() == key).then(|| fp.trim().to_owned())
        })
}

impl HostKeyStore for FileHostKeyStore {
    fn verify(&self, host: &str, port: u16, fingerprint: &str) -> HostKeyVerdict {
        let key = format!("{host}:{port}");
        match self.stored_fingerprint(&key) {
            None => HostKeyVerdict::Unknown,
            Some(stored) if stored == fingerprint => HostKeyVerdict::TrustedMatch,
            Some(stored) => HostKeyVerdict::Changed { stored_fp: stored },
        }
    }

    fn trust(&self, host: &str, port: u16, fingerprint: &str) -> Result<(), ServiceError> {
        let key = format!("{host}:{port}");
        let line = format!("{key} {fingerprint}\n");

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ServiceError::Other(format!("create {}: {e}", parent.display())))?;
        }

        let updated = match fs::read_to_string(&self.path) {
            Ok(text) => {
                let mut out = String::with_capacity(text.len() + line.len());
                let mut replaced = false;
                for l in text.lines() {
                    if l.split_once(' ').is_some_and(|(h, _)| h.trim() == key) {
                        if !replaced {
                            out.push_str(&line);
                            replaced = true;
                        }
                    } else {
                        out.push_str(l);
                        out.push('\n');
                    }
                }
                if !replaced {
                    out.push_str(&line);
                }
                out
            }
            // Missing file: start from an empty store.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => line,
            // Any other read failure must not silently wipe existing entries.
            Err(e) => {
                return Err(ServiceError::Other(format!(
                    "read {}: {e}",
                    self.path.display()
                )))
            }
        };

        fs::write(&self.path, updated)
            .map_err(|e| ServiceError::Other(format!("write {}: {e}", self.path.display())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_services::{HostKeyStore, HostKeyVerdict};

    #[test]
    fn unknown_then_trusted_then_changed() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileHostKeyStore::new(dir.path().join("known_hosts"));
        assert_eq!(
            store.verify("nas", 22, "SHA256:aaa"),
            HostKeyVerdict::Unknown
        );
        store.trust("nas", 22, "SHA256:aaa").unwrap();
        assert_eq!(
            store.verify("nas", 22, "SHA256:aaa"),
            HostKeyVerdict::TrustedMatch
        );
        match store.verify("nas", 22, "SHA256:bbb") {
            HostKeyVerdict::Changed { stored_fp } => assert_eq!(stored_fp, "SHA256:aaa"),
            v => panic!("{v:?}"),
        }
    }

    /// Regression: trust() must match host:port exactly, so "nas:22" and
    /// "nas:2222" are independent entries (22 is a string prefix of 2222).
    #[test]
    fn trust_does_not_clobber_similar_host_keys() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileHostKeyStore::new(dir.path().join("known_hosts"));

        // Overwriting "nas:22" must not delete the "nas:2222" entry.
        store.trust("nas", 2222, "SHA256:x").unwrap();
        store.trust("nas", 22, "SHA256:aaa").unwrap();
        assert_eq!(
            store.verify("nas", 2222, "SHA256:x"),
            HostKeyVerdict::TrustedMatch
        );
        assert_eq!(
            store.verify("nas", 22, "SHA256:aaa"),
            HostKeyVerdict::TrustedMatch
        );

        // Reverse direction: overwriting "nas:2222" must not delete "nas:22".
        store.trust("nas", 2222, "SHA256:y").unwrap();
        assert_eq!(
            store.verify("nas", 22, "SHA256:aaa"),
            HostKeyVerdict::TrustedMatch
        );
        assert_eq!(
            store.verify("nas", 2222, "SHA256:y"),
            HostKeyVerdict::TrustedMatch
        );
    }
}
