use std::sync::{Arc, Mutex};

use ace_av::age_package::live_session::{WidgetProgram, WidgetSession};
use ace_core::source::age_package::{
    compile_age_package_widget_runtime, AgePackageCompilation, AgePackageComponent,
    AgePackageDescription, AgeWidgetProperty, AgeWidgetRefs, WIDGET_CONTEXT_BINDING,
};
use ace_core::source::packages::PACKAGES_CAPABILITY;
use auto_lang_core::file_session::FileSession;

use super::{
    file_open, frame, frame_limits, AceStateValue, FileDirectoryBinding, FileMutationBinding, FileOpenBinding,
    LegacyWidgetAlias, PackageFrame, PackageModuleInput,
};

pub(super) struct LivePackage {
    description: AgePackageDescription,
    widget_refs: AgeWidgetRefs,
    program: Arc<WidgetProgram>,
    root: usize,
    timers: usize,
    live_children: bool,
}

pub(super) struct MountedWidget(WidgetSession);

pub(super) struct LiveEvent<'a> {
    pub instance: &'a [usize],
    pub message: &'a str,
    pub args: &'a [AceStateValue],
}

// A mount owns all of its AV cells and arena handles. Its shared program is
// immutable, and no AV reference leaves the lock protecting the mount.
unsafe impl Send for MountedWidget {}

impl LivePackage {
    pub(super) fn prepare(
        units: &[PackageModuleInput],
        entry: &str,
        alias: LegacyWidgetAlias,
        ordinary: &AgePackageCompilation,
        component: &AgePackageComponent,
    ) -> Result<Option<Self>, String> {
        if component.fields.is_empty() || !component.params.is_empty() {
            return Ok(None);
        }
        let mut requires_child_state = false;
        let uses_context = |node: &AgePackageComponent| {
            node.members.iter().any(|row| row.role == "bind" && row.name == WIDGET_CONTEXT_BINDING)
        };
        let mut needs_current_models =
            component.emits_outbound || component.has_watches || component.requests_focus || uses_context(component);
        for site in ordinary.widget_refs.values().flatten() {
            let child = ordinary
                .description
                .components
                .iter()
                .find(|node| node.module == site.module && node.name == site.node)
                .ok_or("unknown checked child widget")?;
            requires_child_state |= !child.fields.is_empty()
                || child.handler_msgs.len() != child.handler_forwards.len()
                || site
                    .properties
                    .iter()
                    .any(|property| !matches!(property, AgeWidgetProperty::Message { .. }));
            needs_current_models |= child.emits_outbound || child.has_watches || child.requests_focus || uses_context(child);
        }
        if requires_child_state && !needs_current_models {
            return Ok(None);
        }
        let compiled = match compile_age_package_widget_runtime(
            units,
            entry,
            alias,
            &[PACKAGES_CAPABILITY.into()],
        ) {
            Ok(compiled) => compiled,
            Err(diagnostics)
                if diagnostics.iter().all(|diagnostic| {
                    diagnostic.code.starts_with("source.age.widget-runtime-")
                }) =>
            {
                return Ok(None)
            }
            Err(diagnostics) => return Err(super::package_diagnostic(&diagnostics)),
        };
        let root = compiled
            .description
            .components
            .iter()
            .position(|node| node.module == entry)
            .ok_or("missing checked widget root")?;
        let live_children = compiled.widget_runtime.iter().any(|runtime| {
            runtime.outbound || runtime.focus || runtime.context_handler.is_some() || !runtime.watches.is_empty()
        });
        if requires_child_state && !live_children {
            return Ok(None);
        }
        let timers = compiled
            .widget_runtime
            .iter()
            .find(|runtime| runtime.node == root)
            .ok_or("missing checked widget runtime")?
            .timers
            .len();
        let program = WidgetProgram::prepare(&compiled, entry)?;
        Ok(Some(Self {
            description: compiled.description,
            widget_refs: compiled.widget_refs,
            program,
            root,
            timers,
            live_children,
        }))
    }

    pub(super) fn mount(
        &self,
        generation: u64,
        preferences: &[auto_ui_dialect::preferences::PreferenceBinding],
    ) -> Result<Arc<Mutex<MountedWidget>>, String> {
        let limits = frame_limits();
        let session = self.program.open_with_viewport(generation, viewport_width(), &limits)?;
        for (message, value) in super::preferences::restore(preferences) {
            session.dispatch(message, &[ace_av::age_component::ComponentFieldValue::Str(value)], &limits)?;
        }
        Ok(Arc::new(Mutex::new(MountedWidget(session))))
    }

    fn frame(&self, session: &WidgetSession, revision: u64) -> Result<PackageFrame, String> {
        let limits = frame_limits();
        let view = session.package_view(&self.description, &self.widget_refs, &limits)?;
        let states = states(session)?;
        let timers = (0..self.timers)
            .map(|index| session.timer_enabled(index, &limits))
            .collect::<Result<Vec<_>, _>>()?;
        let mut frame = frame::assemble(
            &self.description,
            &self.widget_refs,
            self.root,
            revision,
            view,
            states,
            timers,
            self.live_children,
        )?;
        frame.focus_requests = session.take_focus_requests();
        Ok(frame)
    }

    pub(super) fn update(
        &self,
        mount: &Mutex<MountedWidget>,
        revision: u64,
        event: Option<LiveEvent<'_>>,
        file: Option<&FileSession>,
        directory: Option<&FileDirectoryBinding>,
        mutation: Option<&FileMutationBinding>,
        open: Option<&FileOpenBinding>,
    ) -> Result<PackageFrame, String> {
        let mounted = mount
            .lock()
            .map_err(|_| "checked widget mount is unavailable")?;
        let session = &mounted.0;
        session.set_viewport_width(viewport_width());
        session.transaction(|| {
            let limits = frame_limits();
            if let Some(event) = event {
                let args = event.args.iter().map(AceStateValue::to_field_value).collect::<Vec<_>>();
                if event.instance.is_empty() {
                    if mutation.is_some_and(|binding| binding.reply_name() == event.message)
                        || directory.is_some_and(|binding| binding.reply_name() == event.message)
                    {
                        return Err("ace.desktop.file-reply: only the file binding can deliver its reply".into());
                    }
                    session.dispatch(event.message, &args, &limits)?;
                } else {
                    session.dispatch_child(event.instance, event.message, &args, &limits)?;
                }
            }
            let current = states(session)?;
            if let Some(binding) = mutation {
                if let Some(action) = binding.pending_request(&current)? {
                    let access = file.ok_or("ace.desktop.file-session: no live mutation binding")?;
                    let prepare = |ok| session.preview_update(|| {
                        session.dispatch(binding.reply_name(), &[ace_av::age_component::ComponentFieldValue::Bool(ok)], &limits)?;
                        let frame = self.frame(session, revision)?;
                        if binding.pending_request(&frame.states)?.is_some() {
                            return Err("ace.desktop.file-reply: mutation reply must clear the pending request".into());
                        }
                        Ok(frame)
                    });
                    let accepted = prepare(true)?;
                    let refused = prepare(false)?;
                    let committed = if action.execute(access) { accepted.commit() } else { refused.commit() };

                    // The mutation is committed even if refreshing its directory fails.
                    let refreshed = session.transaction(|| {
                        self.refresh_directory(session, revision, file, directory, &committed.states)
                    });
                    return Ok(match refreshed {
                        Ok(Some(frame)) => frame,
                        Ok(None) => committed,
                        Err(error) => {
                            log::warn!("ace.desktop.file-reply: mutation refresh failed: {error}");
                            committed
                        }
                    });
                }
            }
            if let Some(binding) = open {
                if let Some(path) = binding.pending_path(&current)? {
                    let access = file.ok_or("ace.desktop.file-session: no live open binding")?;
                    let prepare = |status| session.preview_update(|| {
                        session.dispatch(binding.reply_name(), &[ace_av::age_component::ComponentFieldValue::Int(status)], &limits)?;
                        let frame = self.frame(session, revision)?;
                        if binding.pending_path(&frame.states)?.is_some() {
                            return Err("ace.desktop.file-reply: open reply must clear the pending request".into());
                        }
                        Ok(frame)
                    });
                    if !cfg!(windows) {
                        return Ok(prepare(file_open::UNAVAILABLE)?.commit());
                    }
                    let opened = prepare(file_open::OPENED)?;
                    let denied = prepare(file_open::DENIED)?;
                    return Ok(if file_open::open_document(access, path) { opened.commit() } else { denied.commit() });
                }
            }
            if let Some(frame) = self.refresh_directory(session, revision, file, directory, &current)? {
                return Ok(frame);
            }
            self.frame(session, revision)
        })
    }

    fn refresh_directory(
        &self,
        session: &WidgetSession,
        revision: u64,
        file: Option<&FileSession>,
        binding: Option<&FileDirectoryBinding>,
        current: &[(String, AceStateValue)],
    ) -> Result<Option<PackageFrame>, String> {
        let Some(binding) = binding else {
            return Ok(None);
        };
        let Some(path) = binding.pending_path(current)? else {
            return Ok(None);
        };
        let access = file.ok_or("ace.desktop.file-session: no live directory binding")?;
        let listing = FileDirectoryBinding::list(access, path);
        session.directory_reply(
            binding.reply_name(),
            binding.host_reply(listing),
            &frame_limits(),
        )?;
        let frame = self.frame(session, revision)?;
        if binding.pending_path(&frame.states)?.is_some() {
            return Err(
                "ace.desktop.file-reply: directory reply must clear the pending request".into(),
            );
        }
        Ok(Some(frame))
    }
}

fn viewport_width() -> i32 {
    crate::ui::style::theme::window_width().round() as i32
}

fn states(session: &WidgetSession) -> Result<Vec<(String, AceStateValue)>, String> {
    Ok(session
        .scalar_states()?
        .into_iter()
        .map(|(name, value)| (name, AceStateValue::from(&value)))
        .collect())
}
