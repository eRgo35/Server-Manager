//! Secret stores for the `infra` layer.
//!
//! M1 ships two secret modes: plaintext (values in the config file, named via
//! [`secret_key`]) and prompt (session-lifetime, [`InMemorySecretStore`]).
//! OS-keyring-backed storage is deferred beyond M1 (user decision 2026-09-11).

use std::collections::HashMap;
use std::sync::Mutex;

use sm_core::MachineId;
use sm_services::{SecretKind, SecretStore, ServiceError};

/// Stable storage name for a secret kind (plaintext config files).
pub fn secret_key(kind: SecretKind) -> &'static str {
    match kind {
        SecretKind::SshPassword => "ssh_password",
        SecretKind::KeyPassphrase => "key_passphrase",
        SecretKind::SudoPassword => "sudo_password",
    }
}

/// Process-lifetime in-memory secret store (`prompt` secret mode).
pub struct InMemorySecretStore {
    map: Mutex<HashMap<(String, SecretKind), String>>,
}

impl Default for InMemorySecretStore {
    fn default() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
}

impl SecretStore for InMemorySecretStore {
    fn get(&self, machine_id: &MachineId, kind: SecretKind) -> Option<String> {
        self.map
            .lock()
            .unwrap()
            .get(&(machine_id.to_string(), kind))
            .cloned()
    }

    fn set(&self, machine_id: &MachineId, kind: SecretKind, value: &str) -> Result<(), ServiceError> {
        self.map
            .lock()
            .unwrap()
            .insert((machine_id.to_string(), kind), value.to_owned());
        Ok(())
    }

    fn clear(&self, machine_id: &MachineId, kind: SecretKind) -> Result<(), ServiceError> {
        self.map.lock().unwrap().remove(&(machine_id.to_string(), kind));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_services::{SecretStore, SecretKind};

    #[test]
    fn in_memory_roundtrip() {
        let s = InMemorySecretStore::default();
        let id = sm_core::MachineId::from("nas");
        assert!(s.get(&id, SecretKind::SshPassword).is_none());
        s.set(&id, SecretKind::SshPassword, "hunter2").unwrap();
        assert_eq!(s.get(&id, SecretKind::SshPassword).as_deref(), Some("hunter2"));
        s.clear(&id, SecretKind::SshPassword).unwrap();
        assert!(s.get(&id, SecretKind::SshPassword).is_none());
    }

    /// Clearing an absent key must be Ok, not an error.
    #[test]
    fn in_memory_clear_missing_is_ok() {
        let s = InMemorySecretStore::default();
        s.clear(&sm_core::MachineId::from("ghost"), SecretKind::SudoPassword)
            .unwrap();
    }

    /// Stable names for plaintext config files.
    #[test]
    fn secret_key_names() {
        assert_eq!(secret_key(SecretKind::SshPassword), "ssh_password");
        assert_eq!(secret_key(SecretKind::KeyPassphrase), "key_passphrase");
        assert_eq!(secret_key(SecretKind::SudoPassword), "sudo_password");
    }
}