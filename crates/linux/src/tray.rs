use crate::dbus::{DbusService, ItemState, MenuState, TrayEvent};
use crate::icon::Icon;
use gpui::{Action, MenuItem, MouseButton, Point};
use gpui_tray_core::platform_trait::PlatformTray;
use gpui_tray_core::{BackendError, ClickEvent, Error, Result, RuntimeEvent, Tray};
use log::{debug, error};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

enum BackendCommand {
    SetTray {
        tray: Tray,
        response: Sender<Result<()>>,
    },
    RemoveTray {
        response: Sender<Result<()>>,
    },
    Shutdown,
}

pub(crate) struct LinuxBackend {
    command_tx: Sender<BackendCommand>,
    event_rx: Mutex<Receiver<RuntimeEvent>>,
}

impl LinuxBackend {
    fn send_and_wait(&self, cmd: impl FnOnce(Sender<Result<()>>) -> BackendCommand) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        self.command_tx
            .send(cmd(tx))
            .map_err(|_| Error::Backend(BackendError::ChannelSend))?;

        rx.recv()
            .map_err(|_| Error::Backend(BackendError::ChannelReceive))?
    }
}

impl PlatformTray for LinuxBackend {
    fn set_tray(&self, tray: Tray) -> Result<()> {
        self.send_and_wait(|response| BackendCommand::SetTray { tray, response })
    }

    fn remove_tray(&self) -> Result<()> {
        self.send_and_wait(|response| BackendCommand::RemoveTray { response })
    }

    fn try_recv_event(&self) -> Result<Option<RuntimeEvent>> {
        let rx = self.event_rx.lock().map_err(|_| Error::RuntimeClosed)?;
        match rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(Error::RuntimeClosed),
        }
    }

    fn shutdown(&self) -> Result<()> {
        if self.command_tx.send(BackendCommand::Shutdown).is_err() {
            return Err(Error::RuntimeClosed);
        }
        Ok(())
    }
}

struct WorkerState {
    service: Option<DbusService>,
    item_state: Arc<Mutex<ItemState>>,
    menu_state: Arc<Mutex<MenuState>>,
    menu_actions: HashMap<i32, Box<dyn Action>>,
    current_tray: Option<Tray>,
    tray_event_tx: Sender<TrayEvent>,
}

impl WorkerState {
    fn new(tray_event_tx: Sender<TrayEvent>) -> Self {
        Self {
            service: None,
            item_state: Arc::new(Mutex::new(ItemState {
                title: String::new(),
                tooltip: String::new(),
                icon: None,
            })),
            menu_state: Arc::new(Mutex::new(MenuState::new())),
            menu_actions: HashMap::new(),
            current_tray: None,
            tray_event_tx,
        }
    }

    fn apply_set_tray(&mut self, tray: Tray) -> Result<()> {
        debug!(
            "linux set_tray: visible={}, tooltip={:?}, has_icon={}, has_menu={}",
            tray.visible,
            tray.tooltip,
            tray.icon.is_some(),
            tray.menu_builder.is_some()
        );

        self.current_tray = Some(tray.clone());

        if !tray.visible {
            self.hide_tray();
            return Ok(());
        }

        let had_service = self.service.is_some();

        // Build state first, then publish service. This avoids register/query races.
        self.update_item_state(&tray)?;
        let menu_revision = self.rebuild_menu(&tray)?;
        self.ensure_service()?;

        if had_service {
            let service = self.service.as_ref().ok_or(Error::RuntimeClosed)?;
            service.notify_updated(menu_revision).map_err(|err| {
                Error::Backend(BackendError::platform(
                    "DbusService::notify_updated",
                    err.to_string(),
                ))
            })?;
        }

        Ok(())
    }

    fn apply_remove_tray(&mut self) -> Result<()> {
        if self.current_tray.is_none() {
            return Err(Error::NotFound);
        }

        self.current_tray = None;
        self.hide_tray();
        Ok(())
    }

    fn hide_tray(&mut self) {
        self.service = None;
        self.menu_actions.clear();

        if let Ok(mut item_state) = self.item_state.lock() {
            item_state.icon = None;
        }

        if let Ok(mut menu_state) = self.menu_state.lock() {
            menu_state.clear();
        }
    }

    fn ensure_service(&mut self) -> Result<()> {
        if self.service.is_some() {
            return Ok(());
        }

        let service = DbusService::new(
            self.item_state.clone(),
            self.menu_state.clone(),
            self.tray_event_tx.clone(),
        )
        .map_err(|err| {
            Error::Backend(BackendError::platform("DbusService::new", err.to_string()))
        })?;
        self.service = Some(service);
        Ok(())
    }

    fn update_item_state(&mut self, tray: &Tray) -> Result<()> {
        let mut state = lock_mutex(&self.item_state)?;

        state.tooltip = tray
            .tooltip
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();

        state.title = tray
            .title
            .as_ref()
            .map(ToString::to_string)
            .or_else(|| {
                if state.tooltip.is_empty() {
                    None
                } else {
                    Some(state.tooltip.clone())
                }
            })
            .unwrap_or_else(|| "gpui-tray".to_string());

        state.icon = match tray.icon.as_ref() {
            Some(image) => Some(Icon::from_image(image)?.as_pixmaps().to_vec()),
            None => None,
        };

        debug!(
            "linux item state updated: title='{}', tooltip_len={}, has_icon={}",
            state.title,
            state.tooltip.len(),
            state.icon.is_some()
        );

        Ok(())
    }

    fn rebuild_menu(&mut self, tray: &Tray) -> Result<u32> {
        let mut actions = HashMap::new();
        let revision;
        {
            let mut menu_state = lock_mutex(&self.menu_state)?;
            menu_state.clear();

            if let Some(builder) = tray.menu_builder.as_ref() {
                let items = builder();
                debug!("linux menu rebuild: top-level-items={}", items.len());

                for item in &items {
                    add_menu_item(&mut menu_state, &mut actions, item, 0);
                }
            }

            menu_state.mark_updated();
            revision = menu_state.revision();
        }

        debug!(
            "linux menu actions={}, revision={}",
            actions.len(),
            revision
        );
        self.menu_actions = actions;
        Ok(revision)
    }
}

pub fn create() -> Result<Box<dyn PlatformTray>> {
    let (command_tx, command_rx) = mpsc::channel::<BackendCommand>();
    let (runtime_event_tx, runtime_event_rx) = mpsc::channel::<RuntimeEvent>();
    let (boot_tx, boot_rx) = mpsc::channel::<Result<()>>();

    thread::Builder::new()
        .name("gpui-tray-linux".to_string())
        .spawn(move || {
            backend_thread_main(command_rx, runtime_event_tx, boot_tx);
        })
        .map_err(|err| Error::Backend(BackendError::platform("spawn", err.to_string())))?;

    boot_rx
        .recv()
        .map_err(|_| Error::Backend(BackendError::ChannelReceive))??;

    Ok(Box::new(LinuxBackend {
        command_tx,
        event_rx: Mutex::new(runtime_event_rx),
    }))
}

fn backend_thread_main(
    command_rx: Receiver<BackendCommand>,
    runtime_event_tx: Sender<RuntimeEvent>,
    boot_tx: Sender<Result<()>>,
) {
    let (tray_event_tx, tray_event_rx) = mpsc::channel::<TrayEvent>();
    let mut state = WorkerState::new(tray_event_tx);

    let _ = boot_tx.send(Ok(()));

    let mut running = true;
    while running {
        match command_rx.recv_timeout(Duration::from_millis(12)) {
            Ok(command) => {
                running = handle_command(&mut state, command);

                while let Ok(command) = command_rx.try_recv() {
                    if !handle_command(&mut state, command) {
                        running = false;
                        break;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                running = false;
            }
        }

        while let Ok(event) = tray_event_rx.try_recv() {
            handle_tray_event(&state, event, &runtime_event_tx);
        }
    }

    state.hide_tray();
}

fn handle_command(state: &mut WorkerState, command: BackendCommand) -> bool {
    match command {
        BackendCommand::SetTray { tray, response } => {
            let _ = response.send(state.apply_set_tray(tray));
            true
        }
        BackendCommand::RemoveTray { response } => {
            let _ = response.send(state.apply_remove_tray());
            true
        }
        BackendCommand::Shutdown => false,
    }
}

fn handle_tray_event(
    state: &WorkerState,
    event: TrayEvent,
    runtime_event_tx: &Sender<RuntimeEvent>,
) {
    match event {
        TrayEvent::Activate { x, y } => {
            dispatch_click(runtime_event_tx, MouseButton::Left, x, y);
        }
        TrayEvent::SecondaryActivate { x, y } => {
            dispatch_click(runtime_event_tx, MouseButton::Middle, x, y);
        }
        TrayEvent::ContextMenu { x, y } => {
            dispatch_click(runtime_event_tx, MouseButton::Right, x, y);
        }
        TrayEvent::MenuClicked { id } => {
            if let Some(action) = state.menu_actions.get(&id) {
                debug!("linux menu click id={id}");
                let _ = runtime_event_tx.send(RuntimeEvent::Action(action.boxed_clone()));
            } else {
                error!("linux menu click id={id} had no mapped action");
            }
        }
    }
}

fn dispatch_click(runtime_event_tx: &Sender<RuntimeEvent>, button: MouseButton, x: i32, y: i32) {
    debug!("linux click button={:?}, x={}, y={}", button, x, y);

    let event = ClickEvent {
        button,
        position: Point::new(x as f32, y as f32),
    };

    let _ = runtime_event_tx.send(RuntimeEvent::Action(Box::new(event)));
}

fn add_menu_item(
    menu_state: &mut MenuState,
    actions: &mut HashMap<i32, Box<dyn Action>>,
    item: &MenuItem,
    parent_id: i32,
) {
    match item {
        MenuItem::Separator => {
            menu_state.add_separator(parent_id);
        }
        MenuItem::Action { name, action, .. } => {
            let id = menu_state.add_item(name.to_string(), parent_id);
            actions.insert(id, action.boxed_clone());
        }
        MenuItem::Submenu(submenu) => {
            let id = menu_state.add_item(submenu.name.to_string(), parent_id);
            for child in &submenu.items {
                add_menu_item(menu_state, actions, child, id);
            }
        }
        _ => {}
    }
}

fn lock_mutex<'a, T>(mutex: &'a Mutex<T>) -> Result<MutexGuard<'a, T>> {
    mutex.lock().map_err(|_| Error::RuntimeClosed)
}
