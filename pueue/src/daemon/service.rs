//! How Windows services (and this service) work
//!
//! This service runs as SYSTEM, and survives logoff and logon. On startup/login, it launches
//! pueued as the current user. On logoff Windows kills all user processes (including the daemon).
//!
//! - All install/uninstallations of the service requires you are running as admin.
//! - You must install the service. After installed, the service entry maintains a cmdline string
//!   with args to the pueued binary. Therefore, the binary must _not_ move while the service is
//!   installed, otherwise it will not be able to function properly. It is best not to rely on PATH
//!   for this, as it is finicky and a hassle for user setup. Absolute paths are the way to go, and
//!   it is standard practice.
//! - To move the pueued binary: Uninstall the service, move the binary, and reinstall the service.
//! - When the service is installed, you can use pueued cli to start, stop, or uninstall the
//!   service. You can also use the official service manager to start, stop, and restart the
//!   service.
//! - Services are automatically started/stopped by the system according to the setting the user
//!   sets in the windows service manager. By default we install it as autostart, but the user can
//!   set this to manual or even disabled.
//! - If you have the official service manager window open and you tell pueued to uninstall the
//!   service, it will not disappear from the list until you close all service manager windows. This
//!   is not a bug. It's Windows specific behavior. (In Windows parlance, the service is pending
//!   deletion, and all HANDLES to the service need to be closed).
//! - We do not support long running daemon past when a user logs off; this would be a massive
//!   security risk to allow anyone to launch tasks as SYSTEM. This account bypasses even
//!   administrator in power!
//! - Additionally, taking the above into account, as SYSTEM is its own account, the user config
//!   does not apply to this account. Unless there's an exception for config locations with this
//!   case, you'd have to set up separate configs for the SYSTEM account in
//!   `C:\Windows\system32\config\systemprofile\AppData`. (And even if there's an exception, what if
//!   there's multiple users? Which user's config would be used?) This is very unintuitive.
//! - Is the service failing to start up? It's probably a problem with the daemon itself. Re-run
//!   `pueued` by itself to see the actual startup error.

use std::{
    env,
    ffi::{OsString, c_void},
    iter,
    path::PathBuf,
    ptr,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread,
    time::Duration,
};

use windows::{
    Win32::{
        Foundation::{HANDLE, LUID},
        Security::{
            AdjustTokenPrivileges, DuplicateTokenEx, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
            SE_PRIVILEGE_REMOVED, SE_TCB_NAME, SecurityIdentification, TOKEN_ACCESS_MASK,
            TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TokenPrimary,
        },
        System::{
            Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock},
            RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken},
            SystemServices::MAXIMUM_ALLOWED,
            Threading::{
                CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, CreateProcessAsUserW,
                GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken,
                PROCESS_INFORMATION, STARTUPINFOW, TerminateProcess, WaitForSingleObject,
            },
        },
    },
    core::{Free, PCWSTR, PWSTR},
};
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
        SessionChangeParam, SessionChangeReason, SessionNotification,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::internal_prelude::*;

#[derive(Clone)]
struct Config {
    config_path: Option<PathBuf>,
    profile: Option<String>,
}

// The name of the installed service.
const SERVICE_NAME: &str = "pueued";
// The type of service. This one runs in its own dedicated process.
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
// This static lets us communicate Config over ffi callbacks.
static CONFIG: OnceLock<Config> = OnceLock::new();

// For how this works, please see docs @
// <https://docs.rs/windows-service/0.7.0/windows_service/#basics>
define_windows_service!(ffi_service_main, service_main);

/// The main service callback after `ffi_service_main`.
fn service_main(_: Vec<OsString>) {
    if let Err(e) = event_loop() {
        error!("Failed to start service: {e}");
    }
}

/// Installs the service.
///
/// This must be run as admin.
///
/// This passes the config and profile flags passed at the time of install to the service, e.g.
/// `pueued --config my-path --profile my_profile service install`
/// becomes ->
/// `C:\path\pueued.exe --config "my-path" --profile "my_profile" service run`
///
/// This is set to run as SYSTEM user, and survives login/logoffs.
pub fn install_service(config_path: Option<PathBuf>, profile: Option<String>) -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_binary_path = std::env::current_exe()?;

    let mut args = vec![];
    if let Some(config_path) = config_path {
        args.extend([
            "--config".into(),
            format!(r#""{}""#, config_path.to_string_lossy()).into(),
        ]);
    }
    if let Some(profile) = profile {
        args.extend(["--profile".into(), format!(r#""{profile}""#).into()]);
    }

    args.extend(["service".into(), "run".into()]);

    let service_info = ServiceInfo {
        name: SERVICE_NAME.into(),
        display_name: SERVICE_NAME.into(),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: args,
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };

    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description("pueued daemon is a task management tool for sequential and parallel execution of long-running tasks.")?;

    Ok(())
}

/// Uninstall the service.
///
/// This must be run as admin.
pub fn uninstall_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // The service will be marked for deletion as long as this function call succeeds.
    // However, it will not be deleted from the database until it is stopped and all open handles to
    // it are closed. If the service manager window is open, it will need to be closed before
    // the service gets deleted.
    service.delete()?;

    // Our handle to it is not closed yet. So we can still query it.
    if service.query_status()?.current_state != ServiceState::Stopped {
        // If the service cannot be stopped, it will be deleted when the system restarts.
        service.stop()?;
    }

    Ok(())
}

/// Start the service.
///
/// This can also be done from the windows service manager.
pub fn start_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::START;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    match service.query_status()?.current_state {
        ServiceState::Stopped => {
            service.start::<String>(&[])?;
            println!("Successfully started service");
        }
        ServiceState::StartPending => eprintln!("Service is already starting"),
        ServiceState::Running => eprintln!("Service is already running"),

        _ => (),
    }

    Ok(())
}

/// Stop the service.
///
/// This can also be done from the windows service manager.
pub fn stop_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    match service.query_status()?.current_state {
        ServiceState::Stopped => eprintln!("Service is already stopped"),
        ServiceState::StartPending => eprintln!(
            "Service cannot stop because it is starting (please wait until it fully started to stop it)"
        ),
        ServiceState::Running => {
            service.stop()?;
            println!("Successfully stopped service");
        }

        _ => (),
    }

    Ok(())
}

/// Begins running the pueued service.
///
/// This calls `ffi_service_main` -> `service_main` -> `event_loop`
pub fn run_service(config_path: Option<PathBuf>, profile: Option<String>) -> Result<()> {
    CONFIG
        .set(Config {
            config_path,
            profile,
        })
        .map_err(|_| eyre!("static CONFIG set failed"))?;

    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

/// This is the main event loop for the service.
///
/// This gets called from `run_service` -> `ffi_service_main` -> `service_main` -> `event_loop`
fn event_loop() -> Result<()> {
    let spawner = Arc::new(Spawner::new());
    // Whether a shutdown of the service was requested.
    let shutdown = Arc::new(AtomicBool::default());

    // The main event handler for the service.
    let event_handler = {
        let spawner = spawner.clone();
        let shutdown = shutdown.clone();

        move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Stop
                ServiceControl::Stop => {
                    debug!("event stop");
                    // Important! Set the while loop's exit condition before calling stop(),
                    // otherwise the condition will not be observed.
                    shutdown.store(true, Ordering::Relaxed);
                    spawner.stop();

                    ServiceControlHandlerResult::NoError
                }

                // Logon
                ServiceControl::SessionChange(SessionChangeParam {
                    reason: SessionChangeReason::SessionLogon,
                    notification:
                        SessionNotification {
                            session_id: session,
                            ..
                        },
                }) => {
                    debug!("event login");
                    if !spawner.running() {
                        debug!("event login: spawning");
                        if let Err(e) = spawner.start(session) {
                            error!("failed to spawn: {e}");
                            return ServiceControlHandlerResult::Other(1);
                        }
                    }

                    ServiceControlHandlerResult::NoError
                }

                // Logoff
                ServiceControl::SessionChange(SessionChangeParam {
                    reason: SessionChangeReason::SessionLogoff,
                    ..
                }) => {
                    // Windows kills all user processes on logoff.
                    // So we don't actually need to stop any running spawner.
                    debug!("event logoff");

                    ServiceControlHandlerResult::NoError
                }

                // Other session change events we don't care about.
                ServiceControl::SessionChange(_) => ServiceControlHandlerResult::NoError,

                // All services must accept Interrogate even if it's a no-op.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                // Nothing else is implemented.
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    let set_status = move |state: ServiceState, controls: ServiceControlAccept| -> Result<()> {
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: state,
            controls_accepted: controls,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    };

    set_status(ServiceState::StartPending, ServiceControlAccept::empty())?;

    // Make sure we have privileges - this should always succeed
    if let Err(e) = set_privilege(SE_TCB_NAME, true) {
        set_status(ServiceState::Stopped, ServiceControlAccept::empty())?;
        bail!("failed to set privileges: {e}");
    }

    // This attempt is required in order to properly start pueued if the user starts/restarts
    // the service manually, since no events would have been triggered from that.
    //
    // If we can get the current user session on startup, then try to start the spawner.
    //
    // If we can't get the current session, that's OK. It just means there was no user logged in
    // when the service started.
    //
    // The event handler will start it when the user logs in.
    if let Some(session) = get_current_session() {
        if let Err(e) = spawner.start(session) {
            error!("failed to spawn: {e}");
        }
    }

    set_status(
        ServiceState::Running,
        ServiceControlAccept::STOP | ServiceControlAccept::SESSION_CHANGE,
    )?;

    // While there's no shutdown request, and the spawner didn't exit unexpectedly,
    // keep the service running.
    while !shutdown.load(Ordering::Relaxed) && !spawner.dirty() {
        debug!("spawner wait()");
        spawner.wait();
    }

    info!("shutting down service");

    set_status(ServiceState::Stopped, ServiceControlAccept::empty())?;

    Ok(())
}

/// Set the specified process privilege to state.
/// <https://learn.microsoft.com/en-us/windows/win32/secauthz/privilege-constants>
fn set_privilege(name: PCWSTR, state: bool) -> Result<()> {
    let process: OwnedHandle = unsafe { GetCurrentProcess().into() };

    let mut token: OwnedHandle = OwnedHandle::default();
    unsafe {
        OpenProcessToken(process.0, TOKEN_ADJUST_PRIVILEGES, &mut token.0)?;
    }

    let mut luid = LUID::default();

    unsafe {
        LookupPrivilegeValueW(PCWSTR::null(), name, &mut luid)?;
    }

    let mut tp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        ..Default::default()
    };

    tp.Privileges[0].Luid = luid;

    let attributes = if state {
        SE_PRIVILEGE_ENABLED
    } else {
        SE_PRIVILEGE_REMOVED
    };

    tp.Privileges[0].Attributes = attributes;

    unsafe {
        AdjustTokenPrivileges(token.0, false, Some(&tp), 0, None, None)?;
    }

    Ok(())
}

/// Get the current user session. Only needed when we don't initially have a session id to go by.
fn get_current_session() -> Option<u32> {
    let session = unsafe { WTSGetActiveConsoleSessionId() };

    match session {
        // No session attached.
        // <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-wtsgetactiveconsolesessionid#return-value>
        0xFFFFFFFF => None,
        // Found a session!
        session => Some(session),
    }
}

/// Run closure and supply the currently logged in user's token.
fn run_as<T>(session_id: u32, cb: impl FnOnce(HANDLE) -> Result<T>) -> Result<T> {
    let mut query_token: OwnedHandle = OwnedHandle::default();
    // Obtain the user's primary access token. Requires we are SYSTEM and have SE_TCB_NAME.
    //
    // Make sure to not leak this token anywhere, as it must remain secure.
    // <https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsqueryusertoken>
    unsafe {
        WTSQueryUserToken(session_id, &mut query_token.0)?;
    }

    let mut token = OwnedHandle::default();

    unsafe {
        DuplicateTokenEx(
            query_token.0,
            TOKEN_ACCESS_MASK(MAXIMUM_ALLOWED),
            None,
            SecurityIdentification,
            TokenPrimary,
            &mut token.0,
        )?;
    }

    drop(query_token);

    cb(token.0)
}

/// Newtype over handle which closes the HANDLE on drop.
#[derive(Default)]
struct OwnedHandle(HANDLE);

unsafe impl Send for OwnedHandle {}
unsafe impl Sync for OwnedHandle {}

impl OwnedHandle {
    fn is_valid(&self) -> bool {
        !self.0.is_invalid()
    }
}

impl From<HANDLE> for OwnedHandle {
    fn from(value: HANDLE) -> Self {
        Self(value)
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            self.0.free();
        }
    }
}

/// A child process. Tries to kill the process when dropped.
struct Child(OwnedHandle);

impl Child {
    fn new() -> Self {
        Self(OwnedHandle::default())
    }

    fn kill(&mut self) -> Result<()> {
        if self.0.is_valid() {
            unsafe {
                TerminateProcess(self.0.0, 0)?;
            }

            self.0 = OwnedHandle::default();
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.0 = OwnedHandle::default();
    }
}

impl Default for Child {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        _ = self.kill();
    }
}

/// A users' environment block.
/// <https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-createenvironmentblock>
struct EnvBlock(*mut c_void);

impl EnvBlock {
    /// get the environment block belonging to the supplied users token
    fn new(token: HANDLE) -> Result<Self> {
        let mut env = ptr::null_mut();
        unsafe {
            CreateEnvironmentBlock(&mut env, Some(token), false)?;
        }

        Ok(Self(env))
    }
}

impl Drop for EnvBlock {
    fn drop(&mut self) {
        _ = unsafe { DestroyEnvironmentBlock(self.0) };
    }
}

/// Manages the child daemon, by spawning / stopping it, or reporting abnormal exit, and allowing
/// wait().
struct Spawner {
    // Whether a child daemon is running.
    running: Arc<AtomicBool>,
    // Holds the actual process of the running child daemon.
    child: Arc<Mutex<Child>>,
    // Whether the process has exited without our request.
    dirty: Arc<AtomicBool>,
    // Used to differentiate between requested stop() and if process is dirty (see above).
    request_stop: Arc<AtomicBool>,
    // Used for wait()ing until the child is done.
    wait_tx: Sender<()>,
    // We don't need mutation, but we do need Sync.
    wait_rx: Mutex<Receiver<()>>,
}

impl Spawner {
    fn new() -> Self {
        let (wait_tx, wait_rx) = channel();

        Self {
            running: Arc::default(),
            child: Arc::default(),
            dirty: Arc::default(),
            request_stop: Arc::default(),
            wait_tx,
            wait_rx: Mutex::new(wait_rx),
        }
    }

    /// Is the child daemon running?
    fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop the spawned daemon.
    ///
    /// Note: if you need any `while` loop to exit by checking a condition,
    /// make _sure_ you put this stop() _after_ you change the `while` condition to false
    /// otherwise any condition change will not be observed.
    fn stop(&self) {
        let mut child = self.child.lock().unwrap();

        // Request a normal stop. This is not an abnormal process exit.
        self.request_stop.store(true, Ordering::Relaxed);
        match child.kill() {
            Ok(_) => {
                debug!("stop() kill");
                self.running.store(false, Ordering::Relaxed);
                // Signal the wait() to exit so a `while` condition is checked at least once more.
                // As long as `while` conditions have been changed _before_ the call to stop(),
                // the changed condition will be observed.
                _ = self.wait_tx.send(());
            }

            Err(e) => {
                self.running.store(false, Ordering::Relaxed);
                error!("failed to stop(): {e}");
            }
        }
    }

    /// Wait for child process to exit.
    fn wait(&self) {
        _ = self.wait_rx.lock().unwrap().recv();
    }

    /// Did the spawned process quit without our request?
    fn dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    /// Try to spawn a child daemon.
    fn start(&self, session: u32) -> Result<()> {
        let running = self.running.clone();
        let child = self.child.clone();
        let waiter = self.wait_tx.clone();
        let dirty = self.dirty.clone();
        let request_stop = self.request_stop.clone();
        _ = thread::spawn(move || {
            request_stop.store(false, Ordering::Relaxed);

            let res = run_as(session, move |token| {
                let mut arguments = Vec::new();

                let config = CONFIG
                    .get()
                    .ok_or_else(|| eyre!("failed to get CONFIG"))?
                    .clone();

                if let Some(config) = &config.config_path {
                    arguments.push("--config".to_string());
                    arguments.push(format!(r#""{}""#, config.to_string_lossy().into_owned()));
                }

                if let Some(profile) = &config.profile {
                    arguments.push("--profile".to_string());
                    arguments.push(format!(r#""{profile}""#));
                }

                let arguments = arguments.join(" ");

                // Try to get the path to the current binary
                let mut current_exe = env::current_exe()?
                    .to_string_lossy()
                    .to_string()
                    .encode_utf16()
                    .chain(iter::once(0))
                    .collect::<Vec<_>>();

                let mut arguments = arguments
                    .encode_utf16()
                    .chain(iter::once(0))
                    .collect::<Vec<_>>();

                let env_block = EnvBlock::new(token)?;

                let mut process_info = PROCESS_INFORMATION::default();
                unsafe {
                    CreateProcessAsUserW(
                        Some(token),
                        PWSTR(current_exe.as_mut_ptr()),
                        Some(PWSTR(arguments.as_mut_ptr())),
                        None,
                        None,
                        false,
                        // CREATE_UNICODE_ENVIRONMENT is required if we pass env block.
                        // <https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-createenvironmentblock#remarks>
                        //
                        // CREATE_NO_WINDOW causes all child processes to not show a visible
                        // console window. <https://stackoverflow.com/a/71364777/9423933>
                        CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW,
                        Some(env_block.0),
                        None,
                        &STARTUPINFOW::default(),
                        &mut process_info,
                    )?;
                }

                // It is safe to drop this after calling CreateProcessAsUser.
                // <https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-createenvironmentblock#remarks>
                drop(env_block);

                // Store the child process.
                {
                    let mut lock = child.lock().unwrap();
                    *lock = Child(process_info.hProcess.into());
                }

                running.store(true, Ordering::Relaxed);

                // Wait until the process exits.
                unsafe {
                    WaitForSingleObject(process_info.hProcess, INFINITE);
                }

                running.store(false, Ordering::Relaxed);

                // Check if process exited on its own without our explicit request (`stop()` was not
                // called).
                if !request_stop.swap(false, Ordering::Relaxed) {
                    let mut code = 0u32;
                    unsafe {
                        GetExitCodeProcess(process_info.hProcess, &mut code)?;
                    }

                    debug!("spawner code {code}");

                    // Windows gives exit code 0x40010004 when it did a forced process shutdown.
                    // This happens on logoff, so we ignore this code as it's not dirty.
                    if code != 0x40010004 {
                        debug!("service storing dirty true");
                        dirty.store(true, Ordering::Relaxed);
                        _ = waiter.send(());
                    }
                }

                child.lock().unwrap().reset();

                Ok(())
            });

            if let Err(e) = res {
                error!("spawner failed: {e}");
            }
        });

        Ok(())
    }
}
