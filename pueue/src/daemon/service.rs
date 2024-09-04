use std::{
    env,
    ffi::{c_void, OsString},
    iter,
    path::PathBuf,
    process, ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use anyhow::{anyhow, bail, Result};
use log::{debug, error};
use once_cell::sync::OnceCell;
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HANDLE, LUID},
        Security::{
            AdjustTokenPrivileges, DuplicateTokenEx, LookupPrivilegeValueW, SecurityIdentification,
            TokenPrimary, SE_PRIVILEGE_ENABLED, SE_PRIVILEGE_REMOVED, SE_TCB_NAME,
            TOKEN_ACCESS_MASK, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
        },
        System::{
            Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock},
            RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken},
            SystemServices::MAXIMUM_ALLOWED,
            Threading::{
                CreateProcessAsUserW, GetExitCodeProcess, OpenProcess, OpenProcessToken,
                TerminateProcess, WaitForSingleObject, CREATE_NO_WINDOW,
                CREATE_UNICODE_ENVIRONMENT, INFINITE, PROCESS_INFORMATION,
                PROCESS_QUERY_INFORMATION, STARTUPINFOW,
            },
        },
    },
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

#[derive(Clone)]
struct Config {
    config_path: Option<PathBuf>,
    profile: Option<String>,
}

const SERVICE_NAME: &str = "pueued";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
static CONFIG: OnceCell<Config> = OnceCell::new();

define_windows_service!(ffi_service_main, service_main);

pub fn run_service(config_path: Option<PathBuf>, profile: Option<String>) -> Result<()> {
    CONFIG
        .set(Config {
            config_path,
            profile,
        })
        .map_err(|_| anyhow!("static CONFIG set failed"))?;

    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

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

pub fn uninstall_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // The service will be marked for deletion as long as this function call succeeds.
    // However, it will not be deleted from the database until it is stopped and all open handles to it are closed.
    // If the service manager window is open, it will need to be closed before the service gets deleted.
    service.delete()?;

    // Our handle to it is not closed yet. So we can still query it.
    if service.query_status()?.current_state != ServiceState::Stopped {
        // If the service cannot be stopped, it will be deleted when the system restarts.
        service.stop()?;
    }

    Ok(())
}

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
        ServiceState::StartPending => println!("Service is already starting"),
        ServiceState::Running => println!("Service is already running"),

        _ => (),
    }

    Ok(())
}

pub fn stop_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    match service.query_status()?.current_state {
        ServiceState::Stopped => println!("Successfully is already stopped"),
        ServiceState::StartPending => println!("Service cannot stop because it is starting (please wait until it fully started to stop it)"),
        ServiceState::Running => {
            service.stop()?;
            println!("Successfully stopped service");
        }

        _ => (),
    }

    Ok(())
}

fn service_main(_: Vec<OsString>) {
    if let Err(e) = service_event_loop() {
        error!("Failed to start service: {e}");
    }
}

fn service_event_loop() -> Result<()> {
    let spawner = Arc::new(Spawner::new());
    // a shutdown of the service was requested
    let shutdown = Arc::new(AtomicBool::default());

    let event_handler = {
        let spawner = spawner.clone();
        let shutdown = shutdown.clone();

        move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Stop
                ServiceControl::Stop => {
                    debug!("event stop");
                    // important, set the while loop's exit condition before calling stop(), otherwise
                    // the condition will not be observed.
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
                        if let Err(e) = spawner.start(Some(session)) {
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
                    debug!("event logoff");
                    spawner.stop();

                    ServiceControlHandlerResult::NoError
                }

                // Other session change events we don't care about
                ServiceControl::SessionChange(_) => ServiceControlHandlerResult::NoError,

                // All services must accept Interrogate even if it's a no-op.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

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

    // make sure we have privileges - this should always succeed
    if let Err(e) = set_privilege(SE_TCB_NAME, true) {
        set_status(ServiceState::Stopped, ServiceControlAccept::empty())?;
        bail!("failed to set privileges: {e}");
    }

    // if it fails here, we probably launched before user logged in?
    // for that reason we only log, but do not bail and stop the service
    // the event handler will start it when the user logs in
    if let Err(e) = spawner.start(None) {
        error!("failed to spawn: {e}");
    }

    set_status(
        ServiceState::Running,
        ServiceControlAccept::STOP | ServiceControlAccept::SESSION_CHANGE,
    )?;

    // while there's no shutdown request, and the spawner didn't exit unexpectedly,
    // keep the service running
    while !shutdown.load(Ordering::Relaxed) && !spawner.dirty() {
        debug!("spawner wait()");
        spawner.wait();
    }

    debug!("shutting down service");

    set_status(ServiceState::Stopped, ServiceControlAccept::empty())?;

    Ok(())
}

/// set the specified process privilege to state
/// https://learn.microsoft.com/en-us/windows/win32/secauthz/privilege-constants
fn set_privilege(name: PCWSTR, state: bool) -> Result<()> {
    let handle: OwnedHandle =
        unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, false, process::id())?.into() };

    let mut token: OwnedHandle = OwnedHandle::default();
    unsafe {
        OpenProcessToken(handle.0, TOKEN_ADJUST_PRIVILEGES, &mut token.0)?;
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

/// get the current user session, only needed when we don't initially have a session id to go by
fn get_current_session() -> Option<u32> {
    let session = unsafe { WTSGetActiveConsoleSessionId() };

    match session {
        0xFFFFFFFF => None,
        session => Some(session),
    }
}

/// run closure and supply the currently logged in user's token
fn run_as<T>(session_id: u32, cb: impl FnOnce(OwnedHandle) -> Result<T>) -> Result<T> {
    let mut query_token: OwnedHandle = OwnedHandle::default();
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

    let t = cb(token)?;

    Ok(t)
}

/// newtype over handle which closes the HANDLE on drop
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
        if !self.0.is_invalid() {
            _ = unsafe { CloseHandle(self.0) };
        }
    }
}

/// a child process. tries to kill the process when dropped
struct Child(OwnedHandle);

impl Child {
    fn new() -> Self {
        Self(OwnedHandle::default())
    }

    fn kill(&mut self) -> Result<()> {
        if self.0.is_valid() {
            unsafe {
                TerminateProcess(self.0 .0, 0)?;
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

/// a users environment block
/// https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-createenvironmentblock
struct EnvBlock(*mut c_void);

impl EnvBlock {
    /// get the environment block belonging to the supplied users token
    fn new(token: HANDLE) -> Result<Self> {
        let mut env = ptr::null_mut();
        unsafe {
            CreateEnvironmentBlock(&mut env, token, false)?;
        }

        Ok(Self(env))
    }
}

impl Drop for EnvBlock {
    fn drop(&mut self) {
        _ = unsafe { DestroyEnvironmentBlock(self.0) };
    }
}

/// manages the child daemon, by spawning / stopping it, or reporting abnormal exit, and allowing wait()
struct Spawner {
    // whether a child daemon is running
    running: Arc<AtomicBool>,
    // holds the actual process of the running child daemon
    child: Arc<Mutex<Child>>,
    // whether the process has exited without our request
    dirty: Arc<AtomicBool>,
    // used to differentiate between requested stop() and if process is dirty ^
    request_stop: Arc<AtomicBool>,
    // used for wait()ing until the child is done
    wait_tx: Sender<()>,
    // we don't need mutation, but we do need Sync
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

    /// is the child daemon running?
    fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// note: if you need any `while` loop to exit by checking condition,
    /// make _sure_ you put this stop() _after_ you change the `while` condition to false
    /// otherwise it will not be observable
    fn stop(&self) {
        let mut child = self.child.lock().unwrap();

        self.request_stop.store(true, Ordering::Relaxed);
        match child.kill() {
            Ok(_) => {
                debug!("stop() kill");
                self.running.store(false, Ordering::Relaxed);
                // signal the wait() to exit so a `while` condition is checked at least once more.
                // as long as `while` conditions have been changed _before_ the call to stop(),
                // the changed condition will be observed
                _ = self.wait_tx.send(());
            }

            Err(e) => {
                self.running.store(false, Ordering::Relaxed);
                error!("failed to stop(): {e}");
            }
        }
    }

    /// wait for child process to exit
    fn wait(&self) {
        _ = self.wait_rx.lock().unwrap().recv();
    }

    /// did the spawned process quit without our request?
    fn dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    /// try to spawn a child daemon
    fn start(&self, session: Option<u32>) -> Result<()> {
        let Some(session) = session.or_else(get_current_session) else {
            bail!("get_current_session failed");
        };

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
                    .ok_or_else(|| anyhow!("failed to get CONFIG"))?
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
                let current_exe = env::current_exe()?.to_string_lossy().to_string();

                let mut command = format!(r#""{current_exe}" {arguments}"#)
                    .encode_utf16()
                    .chain(iter::once(0))
                    .collect::<Vec<_>>();

                // lpcommandline may modify the the cmd vec. max valid size is 1024, so
                // I think it best to ensure 1024 bytes of capacity total exist just in case
                command.reserve(1024 - (command.len() / 2));

                let env_block = EnvBlock::new(token.0)?;

                let mut process_info = PROCESS_INFORMATION::default();
                unsafe {
                    CreateProcessAsUserW(
                        token.0,
                        None,
                        PWSTR(command.as_mut_ptr()),
                        None,
                        None,
                        false,
                        // unicode is required if we pass env block
                        // create_no_window causes all child processes to not show a visible console window
                        CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW,
                        Some(env_block.0),
                        None,
                        &STARTUPINFOW::default(),
                        &mut process_info,
                    )?;
                }

                {
                    let mut lock = child.lock().unwrap();
                    *lock = Child(process_info.hProcess.into());
                    running.store(true, Ordering::Relaxed);
                }

                // wait until the process exits
                unsafe {
                    WaitForSingleObject(process_info.hProcess, INFINITE);
                }

                running.store(false, Ordering::Relaxed);

                // check if process exited on its own without our explicit request
                if !request_stop.swap(false, Ordering::Relaxed) {
                    let mut code = 0u32;
                    unsafe {
                        GetExitCodeProcess(process_info.hProcess, &mut code)?;
                    }

                    debug!("spawner code {code}");

                    // windows gives this exit code on the process in event of forced process shutdown
                    // this happens on logoff, so we treat this code as normal
                    const LOGOFF: u32 = 0x40010004;
                    if code != 0 && code != LOGOFF {
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
