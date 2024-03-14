use starknet_prove_core::{Proof, ProofRequest, Prove};
use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{ExitStatus, Stdio},
};
use tokio::{
    io::AsyncReadExt,
    process::{ChildStderr, Command},
};

use self::input::write_inputs_to_directory;

mod input;

const PROOF_FILE: &str = "proof_file.json";
const PRIVATE_INPUT_FILE: &str = "private_input_file.json";
const PUBLIC_INPUT_FILE: &str = "public_input_file.json";
const PROVER_CONFIG_FILE: &str = "prover_config.json";
const PARAMETER_FILE: &str = "parameter_file.json";
const MEMORY_FILE: &str = "memory_file.bin";
const TRACE_FILE: &str = "trace_file.bin";

/// An error that can be produced by the [`StoneProver`] implementation of the [`Prove`] trait.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The child process could not be spawned, or any other system-level error occured.
    #[error("{0}")]
    Io(
        #[source]
        #[from]
        io::Error,
    ),
    /// The child process exited with an unexpected error code.
    #[error("{1} (exit code {0})")]
    UnexpectedErrorCode(ExitStatus, String),
    /// An error occured serializing or deserializing JSON input/output.
    ///
    /// This usually occurs when the prover returns an unexpected output.
    #[error("{0}")]
    Serde(
        #[from]
        #[source]
        serde_json::Error,
    ),
}

/// The configuration passed to [`StoneProver`] to configure its behavior.
#[derive(Debug, Clone)]
pub struct StoneConfig {
    /// The working directory in which the Stone prover will be executed.
    ///
    /// This is requiered because [`StoneProver`] invokes a command in the background, and that
    /// commands takes all of its inputs from the file system.
    ///
    /// The inputs of the command are written to the file system at that location.
    ///
    /// # Remarks
    ///
    /// This directory won't be automatically created, so it must exist prior to using the
    /// prover.
    pub working_directory: PathBuf,
    /// The command that will be spawned every time a proof is requested.
    ///
    /// Note that this is relative to `working_directory` (unless the path is absolute or is a
    /// command).
    pub command: OsString,
}

impl Default for StoneConfig {
    fn default() -> Self {
        Self {
            working_directory: ".".into(),
            command: "cpu_air_prover".into(),
        }
    }
}

/// Contains the state required to run the Stone prover in the background and generate proofs with
/// it.
///
/// This type implements the [`Prove`] trait.
///
/// # Child process
///
/// This [`Prove`] implementation spawns a background process that runs the Stone prover.
///
/// ## Security concerns
///
/// Because an external process is being spawned and given access to the file system, it is
/// important to make sure that the *correct* command is being run.
///
/// Currently, no `chroot` or other sandboxing mechanism is being used to run the prover, meaning
/// that if a malicious command is used inadvertedly, it could potentially access or modify the
/// entire file system.
///
/// The environment is cleared to make sure that the prover can't access unnescessary information.
///
/// ## File system assumptions
///
/// When a [`StoneProver`] is created, it is assumed that the working directory exists and that
/// the prover has the necessary permissions to read and write files in that directory. It is
/// expected that the directory is not removed, renamed, or modified in any way while the prover
/// is running.
///
/// Note that the prover will write files to the working directory, and it is important to make
/// sure that those files are not removed while it is running.
pub struct StoneProver {
    /// The working directory in which the Stone prover will be executed.
    working_directory: PathBuf,
    /// The command that will be spawned every time a proof is requested.
    command: Command,
}

impl StoneProver {
    /// Creates a new [`StoneProver`] instance from the provided configuration.
    pub fn new(config: StoneConfig) -> Self {
        let command = make_command(&config);
        let working_directory = config.working_directory;

        Self {
            working_directory,
            command,
        }
    }
}

impl Prove for StoneProver {
    type Err = Error;

    async fn prove(&mut self, request: &ProofRequest<'_>) -> Result<Proof, Self::Err> {
        write_inputs_to_directory(request, &self.working_directory).await?;

        let mut child = self.command.spawn()?;
        let stderr = child.stderr.take().unwrap();
        let status = child.wait().await?;

        // If the command has failed, we read the error message from the standard error stream
        // and return it.
        if !status.success() {
            let error_message = error_message(stderr).await;
            return Err(Error::UnexpectedErrorCode(status, error_message));
        }

        // TODO: Parse `PROOF_FILE`

        Ok(Proof {})
    }
}

/// Returns the [`Command`] that will be used by [`StoneProver`] to spawn the process
/// responsible for generating proofs.
fn make_command(config: &StoneConfig) -> Command {
    let mut command = Command::new(&config.command);

    command
        .current_dir(&config.working_directory)
        .env_clear() // cleared for security
        .arg("--out_file")
        .arg(PROOF_FILE)
        .arg("--private_input_file")
        .arg(PRIVATE_INPUT_FILE)
        .arg("--public_input_file")
        .arg(PUBLIC_INPUT_FILE)
        .arg("--prover-config-file")
        .arg(PROVER_CONFIG_FILE)
        .arg("--parameter_file")
        .arg(PARAMETER_FILE)
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .stderr(Stdio::piped()) // stderr needs to be piped to capture error messages
        .kill_on_drop(true); // ensures that any error occuring before the child is waited on kills it

    command
}

/// Gets the error message stored in the standard error stream of a child process.
async fn error_message(mut stderr: ChildStderr) -> String {
    let mut buf = Vec::new();
    match stderr.read_to_end(&mut buf).await {
        Ok(_) => (),
        Err(_) => return "<failed to read error message>".into(),
    }
    match String::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => "<error message is not valid UTF-8>".into(),
    }
}
