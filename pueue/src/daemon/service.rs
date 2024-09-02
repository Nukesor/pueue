use std::{
    env,
    ffi::{c_void, OsString},
    iter,
    path::PathBuf,
    process, ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, Thread},
    time::Duration,
};

use anyhow::{anyhow, bail, Result};
use log::error;
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
                CreateProcessAsUserW, OpenProcess, OpenProcessToken, TerminateProcess,
                WaitForSingleObject, CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, INFINITE,
                PROCESS_INFORMATION, PROCESS_QUERY_INFORMATION, STARTUPINFOW,
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

pub fn start_service(config_path: Option<PathBuf>, profile: Option<String>) -> Result<()> {
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

    let mut args = vec!["--service".into()];
    if let Some(config_path) = config_path {
        args.push("--config".into());
        args.push(format!(r#""{}""#, config_path.to_string_lossy()).into());
    }
    if let Some(profile) = profile {
        args.push("--profile".into());
        args.push(format!(r#""{profile}""#).into());
    }

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
    service.delete()?;

    // Our handle to it is not closed yet. So we can still query it.
    if service.query_status()?.current_state != ServiceState::Stopped {
        // If the service cannot be stopped, it will be deleted when the system restarts.
        service.stop()?;
    }

    Ok(())
}

fn service_main(_: Vec<OsString>) {
    if let Err(e) = run_service() {
        error!("Failed to start service: {e}");
    }
}

fn run_service() -> Result<()> {
    let spawner = Arc::new(Spawner::new());
    let shutdown = Arc::new(AtomicBool::default());

    let event_handler = {
        let spawner = spawner.clone();
        let shutdown = shutdown.clone();

        move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop => {
                    shutdown.store(true, Ordering::Relaxed);
                    spawner.stop();

                    ServiceControlHandlerResult::NoError
                }

                // Logon
                ServiceControl::SessionChange(SessionChangeParam {
                    reason: SessionChangeReason::SessionLogon,
                    notification: SessionNotification { session_id, .. },
                }) => {
                    if !spawner.running() {
                        if let Err(e) = spawner.start(Some(session_id)) {
                            error!("failed to spawn: {e}");
                        }
                    }

                    ServiceControlHandlerResult::NoError
                }

                // Logoff
                ServiceControl::SessionChange(SessionChangeParam {
                    reason: SessionChangeReason::SessionLogoff,
                    ..
                }) => {
                    spawner.stop();

                    ServiceControlHandlerResult::NoError
                }

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

    // make sure we have privileges
    if let Err(e) = set_privilege(SE_TCB_NAME, true) {
        set_status(ServiceState::Stopped, ServiceControlAccept::empty())?;
        bail!("failed to set privileges: {e}");
    }

    // if it fails here, we probably launched before user logged in?
    // for that reason we only log, but do not bail and stop the service
    if let Err(e) = spawner.start(None) {
        error!("failed to spawn: {e}");
    }

    set_status(
        ServiceState::Running,
        ServiceControlAccept::STOP | ServiceControlAccept::SESSION_CHANGE,
    )?;

    while !shutdown.load(Ordering::Relaxed) && !spawner.dirty() {
        spawner.wait();
    }

    set_status(ServiceState::Stopped, ServiceControlAccept::empty())?;

    Ok(())
}

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

fn get_current_session() -> Option<u32> {
    let session = unsafe { WTSGetActiveConsoleSessionId() };

    match session {
        0xFFFFFFFF => None,
        session => Some(session),
    }
}

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

struct EnvBlock(*mut c_void);

impl EnvBlock {
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

struct Spawner {
    running: Arc<AtomicBool>,
    child: Arc<Mutex<Child>>,
    main: Arc<Thread>,
    // whether the process has exited without our request
    dirty: Arc<AtomicBool>,
    request_stop: Arc<AtomicBool>,
}

impl Spawner {
    fn new() -> Self {
        Self {
            running: Arc::default(),
            child: Arc::default(),
            main: Arc::new(thread::current()),
            dirty: Arc::default(),
            request_stop: Arc::default(),
        }
    }

    fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    // note: if you need any `while` loop to exit by checking condition,
    // make _sure_ you put this stop() _after_ you change the `while` condition to false
    // otherwise it will not be observable
    fn stop(&self) {
        let mut child = self.child.lock().unwrap();
        self.request_stop.store(true, Ordering::Relaxed);
        match child.kill() {
            Ok(_) => {
                self.running.store(false, Ordering::Relaxed);
                // even if thread got stuck in park(), this ensures it will test the
                // `while` condition at least once more. as long as `while` conditions have
                // been changed _before_ the call to stop(), it will exit the wait()
                self.main.unpark();
            }

            Err(e) => {
                error!("failed to stop(): {e}");
            }
        }
    }

    fn wait(&self) {
        let mut handle = self.child.lock().unwrap().0 .0;

        if handle.is_invalid() {
            while !self.running() {
                thread::park();
            }

            handle = self.child.lock().unwrap().0 .0;
        }

        unsafe {
            WaitForSingleObject(handle, INFINITE);
        }
    }

    /// did the spawned process quit without our request?
    fn dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn start(&self, session: Option<u32>) -> Result<()> {
        let Some(session) = session.or_else(get_current_session) else {
            bail!("get_current_session failed");
        };

        let running = self.running.clone();
        let child = self.child.clone();
        let main_thread = self.main.clone();
        let dirty = self.dirty.clone();
        let request_stop = self.request_stop.clone();
        _ = thread::spawn(move || {
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

                // Try to get the path to the current binary, since it may not be in the $PATH.
                // If we cannot detect it (for some unknown reason), fallback to the raw `pueued` binary name.
                let current_exe = env::current_exe()?.to_string_lossy().to_string();

                let mut command = format!(r#""{current_exe}" {arguments}"#)
                    .encode_utf16()
                    .chain(iter::once(0))
                    .collect::<Vec<_>>();

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
                    main_thread.unpark();
                }

                unsafe {
                    WaitForSingleObject(process_info.hProcess, INFINITE);
                }

                {
                    let mut lock = child.lock().unwrap();
                    _ = lock.kill();
                    running.store(false, Ordering::Relaxed);
                }

                // check if process exited on its own without our request
                if !request_stop.swap(false, Ordering::Relaxed) {
                    dirty.store(true, Ordering::Relaxed);
                }

                Ok(())
            });

            if let Err(e) = res {
                error!("spawner failed: {e}");
            }
        });

        Ok(())
    }
}
