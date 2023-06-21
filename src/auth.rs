use std::{fmt, process::Stdio, time::Duration};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::OffsetDateTime;
use tokio::{
    process::Command,
    sync::{Mutex, MutexGuard},
};

#[derive(Deserialize)]
pub struct AuthorizationResult {
    #[serde(with = "http_serde::header_map")]
    metadata: http::HeaderMap,
    #[serde(default, with = "time::serde::rfc3339::option")]
    expiry: Option<OffsetDateTime>,
}

pub struct AuthorizationHook {
    shell: String,
    args: Vec<String>,
    value: Mutex<Option<AuthorizationResult>>,
}

impl AuthorizationHook {
    pub fn new(shell: String) -> Result<Self> {
        let args = split_shell(&shell)?;
        if args.is_empty() {
            bail!("no arguments")
        }

        Ok(AuthorizationHook {
            shell,
            args,
            value: Mutex::new(None),
        })
    }

    pub async fn get_headers(&self) -> Result<http::HeaderMap> {
        let value_lock = self.value.lock().await;

        if let Some(result) = &*value_lock {
            if let Some(expires_at) = &result.expiry {
                if *expires_at - OffsetDateTime::now_utc() > Duration::from_secs(300) {
                    return Ok(result.metadata.clone());
                }
            }
        }

        self.get_header_inner(value_lock).await
    }

    pub async fn get_headers_force(&self) -> Result<http::HeaderMap> {
        let value_lock = self.value.lock().await;
        self.get_header_inner(value_lock).await
    }

    async fn get_header_inner<'a>(
        &self,
        mut lock: MutexGuard<'a, Option<AuthorizationResult>>,
    ) -> Result<http::HeaderMap> {
        let child = Command::new(&self.args[0])
            .args(&self.args[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to run authorization plugin")?;
        tracing::info!(
            "Executing command with process id {}: {}",
            child.id().unwrap_or_default(),
            self.shell
        );

        let output = child
            .wait_with_output()
            .await
            .context("failed to run authorization plugin")?;

        if !output.status.success() {
            let detail = if !output.stderr.is_empty() {
                format!(": {}", String::from_utf8_lossy(&output.stderr).trim())
            } else if !output.stdout.is_empty() {
                format!(": {}", String::from_utf8_lossy(&output.stdout).trim())
            } else {
                String::new()
            };

            bail!("authorization plugin returned {}{}", output.status, detail)
        }

        let result: AuthorizationResult = serde_json::from_slice(&output.stdout)
            .context("authorization plugin returned invalid data")?;

        let header = result.metadata.clone();
        *lock = Some(result);
        Ok(header)
    }

    pub fn shell(&self) -> &str {
        &self.shell
    }
}

impl fmt::Debug for AuthorizationHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizationHook")
            .field("shell", &self.shell)
            .finish()
    }
}

impl Serialize for AuthorizationHook {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.shell)
    }
}

impl<'de> Deserialize<'de> for AuthorizationHook {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let shell = String::deserialize(deserializer)?;
        AuthorizationHook::new(shell).map_err(<D::Error as serde::de::Error>::custom)
    }
}

#[cfg(windows)]
fn split_shell(shell: &str) -> Result<Vec<String>> {
    use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt};

    use windows::{
        core::PCWSTR,
        Win32::{Foundation::HLOCAL, System::Memory::LocalFree, UI::Shell::CommandLineToArgvW},
    };

    unsafe {
        let mut shell_utf16 = Vec::with_capacity(shell.len() * 2 + 1);
        shell_utf16.extend(OsStr::new(shell).encode_wide());
        shell_utf16.push(0);

        let mut num_args: i32 = 0;
        let arg_list = CommandLineToArgvW(PCWSTR(shell_utf16.as_ptr()), &mut num_args);
        if arg_list.is_null() {
            return Err(io::Error::last_os_error().into());
        }

        let mut results = Vec::with_capacity(num_args as usize);
        for i in 0..num_args {
            results.push((*arg_list.offset(i as isize)).to_string().unwrap());
        }

        _ = LocalFree(HLOCAL(arg_list as isize));
        Ok(results)
    }
}

#[cfg(not(windows))]
fn split_shell(shell: &str) -> Result<Vec<String>> {
    shell_words::split(shell).map_err(Into::into)
}
