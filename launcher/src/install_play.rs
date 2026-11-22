//! A Legacy-launcher-like panel on Home: pick an account, a Minecraft version and a loader,
//! then press one button to create the Java instance and play it. All downloads still go
//! through the existing authorization gate in `backend::java_game`.
use super::*;
use crate::new_instance::VersionList;
use crate::plate::alpha;
use crate::shell::{c, Action};
use horizon_launcher::account::{self, Player};
use theme::night;
use horizon_launcher::instances;
use horizon_launcher::loaders::Kind as LoaderKind;
use horizon_launcher::minecraft_versions::{self, Kind as VersionKind};

/// State of the install-and-play panel.
#[derive(Default)]
pub(super) struct InstallPlay {
    /// The chosen Mojang version id; None: the newest release.
    pub(super) version: Option<String>,
    pub(super) versions: VersionList,
    /// None: vanilla; Some: Fabric or Quilt.
    pub(super) loader: Option<LoaderKind>,
    /// The account id to play as, `offline:<name>`, or `game` for the game's own name.
    pub(super) account: Option<String>,
    pub(super) show_versions: bool,
    pub(super) status: Option<String>,
}

impl Launcher {
    /// The identities the account button cycles through.
    fn install_identities(&self) -> Vec<(String, String)> {
        let accounts = account::Accounts::load();
        let mut out = vec![("game".to_string(), tr("Game name (offline)").to_string())];
        for a in &accounts.accounts {
            out.push((a.uuid.clone(), a.name.clone()));
        }
        for n in &accounts.offline {
            out.push((format!("offline:{n}"), n.clone()));
        }
        out
    }

    fn install_current(&self) -> (String, String) {
        let id = self.install_play.account.clone().unwrap_or_else(|| {
            match &self.accounts.player() {
                Player::Microsoft(uuid) => uuid.clone(),
                Player::Offline(name) => format!("offline:{name}"),
                Player::GameName => "game".to_string(),
            }
        });
        self.install_identities()
            .into_iter()
            .find(|(i, _)| *i == id)
            .unwrap_or_else(|| ("game".to_string(), tr("Game name (offline)").to_string()))
    }

    /// Fetch Mojang's version list once, the first time the panel is drawn.
    fn fetch_install_versions(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.install_play.versions, VersionList::NotAsked) {
            return;
        }
        self.install_play.versions = VersionList::Loading;
        let task = cx
            .background_executor()
            .spawn(async { minecraft_versions::load() });
        cx.spawn(async move |this, cx| {
            let list = task.await;
            let _ = this.update(cx, |this, _| {
                this.install_play.versions = match list {
                    Ok(v) => VersionList::Ready(v),
                    Err(e) => VersionList::Failed(e),
                };
            });
        })
        .detach();
    }

    /// Creates the Java instance and starts it. A fresh sign-in is required for downloads;
    /// when no owner is signed in we open the gate instead of starting a download.
    fn install_and_play(&mut self, cx: &mut Context<Self>) {
        let version = self.install_play.version.clone();
        let versions = match &self.install_play.versions {
            VersionList::Ready(v) => Some(v),
            _ => None,
        };
        let Some(version) = version.or_else(|| versions.map(|v| v.latest_release.clone())) else {
            self.install_play.status = Some(tr("Mojang's version list is not available yet.").to_string());
            cx.notify();
            return;
        };
        if self.owner_shown() == Some(false) {
            self.sign_in_to_play();
            cx.notify();
            return;
        }

        let mut accounts = account::Accounts::load();
        let who = match self.install_play.account.clone() {
            Some(id) if id == "game" => Player::GameName,
            Some(id) if id.starts_with("offline:") => {
                Player::Offline(id.trim_start_matches("offline:").to_string())
            }
            Some(uuid) => Player::Microsoft(uuid),
            None => accounts.player(),
        };
        if let Err(e) = accounts.play_as_with(account::store(), who, self.settings.dev_offline) {
            self.install_play.status = Some(e);
            cx.notify();
            return;
        }
        if let Err(e) = accounts.save() {
            self.install_play.status = Some(e);
            cx.notify();
            return;
        }
        self.accounts = accounts;

        let loader = self.install_play.loader;
        let choice = new_instance::JavaChoice {
            kind: if loader.is_some() {
                new_instance::NewKind::Modded
            } else {
                new_instance::NewKind::Vanilla
            },
            loader: loader.unwrap_or(LoaderKind::Fabric),
            version: Some(version.clone()),
            snapshots: false,
            old: false,
        };
        let name = choice.default_name(&version);
        let made = match new_instance::create_java(&choice, &name, &version) {
            Ok(i) => i,
            Err(e) if e.contains("already an instance") => {
                match instances::list().into_iter().find(|i| i.name == name) {
                    Some(i) => i,
                    None => {
                        self.install_play.status = Some(e);
                        cx.notify();
                        return;
                    }
                }
            }
            Err(e) => {
                self.install_play.status = Some(e);
                cx.notify();
                return;
            }
        };
        if let Err(e) = instances::select(&made.id) {
            self.install_play.status = Some(e);
            cx.notify();
            return;
        }
        self.install_play.status = None;
        self.reload_instances();
        self.play();
        cx.notify();
    }

    /// The panel itself, shown on the Overview tab of a Java instance.
    pub(super) fn install_play_panel(&mut self, cx: &mut Context<Self>) -> Div {
        self.fetch_install_versions(cx);
        let t = self.theme;
        let n = t.night;
        let caption = |s: &str| {
            shell::text(night::CAPTION, c(n.muted))
                .font_weight(FontWeight::SEMIBOLD)
                .child(tr(s).to_string())
        };
        let (account_id, account_name) = self.install_current();
        let version = self
            .install_play
            .version
            .clone()
            .or_else(|| match &self.install_play.versions {
                VersionList::Ready(v) => Some(v.latest_release.clone()),
                _ => None,
            })
            .unwrap_or_else(|| tr("latest release").to_string());

        let account_button = shell::action("install-account", Action::Raised, &t)
            .child(shell::label(format!("{}: {}", tr("Account"), account_name)))
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                let ids = this.install_identities();
                let next = ids
                    .iter()
                    .position(|(id, _)| id == &account_id)
                    .map_or(0, |i| (i + 1) % ids.len());
                this.install_play.account = Some(ids[next].0.clone());
                cx.notify();
            }));

        let version_button = shell::action("install-version", Action::Raised, &t)
            .child(shell::label(format!("{}: {}", tr("Version"), version)))
            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                this.install_play.show_versions = !this.install_play.show_versions;
                cx.notify();
            }));

        let mut rows = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(caption("Install and play"))
            .child(div().flex().gap(px(8.)).child(account_button).child(version_button));

        if self.install_play.show_versions {
            let list: Vec<gpui::AnyElement> = match &self.install_play.versions {
                VersionList::Ready(versions) => versions
                    .all
                    .iter()
                    .filter(|v| v.kind == VersionKind::Release)
                    .take(60)
                    .map(|v| {
                        let id = v.id.clone();
                        let selected = self.install_play.version.as_deref() == Some(v.id.as_str());
                        shell::row(format!("install-v-{}", v.id), selected, &t)
                            .flex_none()
                            .h(px(30.))
                            .px(px(12.))
                            .child(shell::text(night::UI, c(n.ink)).child(v.id.clone()))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.install_play.version = Some(id.clone());
                                this.install_play.show_versions = false;
                                cx.notify();
                            }))
                            .into_any_element()
                    })
                    .collect(),
                VersionList::NotAsked | VersionList::Loading => vec![
                    shell::text(night::UI, c(n.muted))
                        .child(tr("Asking Mojang for its versions…").to_string())
                        .into_any_element(),
                ],
                VersionList::Failed(e) => vec![
                    shell::text(night::UI, c(n.danger))
                        .child(e.clone())
                        .into_any_element(),
                ],
            };
            rows = rows.child(
                div()
                    .id("install-versions")
                    .flex()
                    .flex_col()
                    .h(px(180.))
                    .p(px(4.))
                    .overflow_y_scroll()
                    .rounded(px(night::RADIUS_CONTROL))
                    .border_1()
                    .border_color(alpha(n.stroke, n.stroke_alpha))
                    .children(list),
            );
        }

        let loader_buttons = [(None, "Vanilla"), (Some(LoaderKind::Fabric), "Fabric"), (Some(LoaderKind::Quilt), "Quilt")]
            .into_iter()
            .map(|(kind, label)| {
                let on = self.install_play.loader == kind;
                shell::action(
                    format!("install-loader-{label}"),
                    if on { Action::Raised } else { Action::Quiet },
                    &t,
                )
                .child(shell::label(tr(label).to_string()))
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.install_play.loader = kind;
                    cx.notify();
                }))
            });
        rows = rows
            .child(caption("Mode"))
            .child(div().flex().gap(px(8.)).children(loader_buttons))
            .child(
                shell::action("install-play", Action::Main, &t)
                    .child(shell::label(tr("Install and play").to_string()))
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.install_and_play(cx);
                    })),
            );
        if let Some(status) = &self.install_play.status {
            rows = rows.child(
                shell::text(night::CAPTION, c(n.danger))
                    .child(status.clone()),
            );
        }
        rows.child(shell::text(night::CAPTION, c(n.muted)).child(
            tr("The game files are downloaded from Mojang for your account the first time.")
                .to_string(),
        ))
    }
}
