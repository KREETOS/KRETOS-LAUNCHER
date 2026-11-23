//! The first-run setup wizard: choose a Minecraft version, sign in (or go offline), then create
//! the Java instance. It reuses the same version list, Java check and installer as Home.
use super::*;
use crate::new_instance::VersionList;
use horizon_launcher::minecraft_versions::Kind as VersionKind;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    Version,
    Account,
    Ready,
}

pub(super) struct Wizard {
    pub(super) open: bool,
    step: Step,
}

impl Default for Wizard {
    fn default() -> Self {
        Self {
            open: false,
            step: Step::Version,
        }
    }
}

impl Launcher {
    /// Opens the wizard again from Settings.
    pub(super) fn open_wizard(&mut self) {
        self.wizard.open = true;
        self.wizard.step = Step::Version;
    }

    /// The first-run overlay, shown only before any Java instance exists.
    pub(super) fn wizard_page(&mut self, cx: &mut Context<Self>) -> Stateful<Div> {
        self.fetch_install_versions(cx);
        let t = self.theme;
        let n = t.night;
        let title = match self.wizard.step {
            Step::Version => tr("Step 1 of 3: Choose a Minecraft version"),
            Step::Account => tr("Step 2 of 3: Choose an account"),
            Step::Ready => tr("Step 3 of 3: Ready to play"),
        };
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                shell::text(theme::CAPTION, shell::c(n.muted))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(tr("KRETOS Launcher setup").to_string()),
            )
            .child(
                shell::text(theme::TITLE, shell::c(n.ink))
                    .child(tr("Welcome to KRETOS Launcher").to_string()),
            )
            .child(shell::text(theme::UI, shell::c(n.ink_soft)).child(title.to_string()));

        body = match self.wizard.step {
            Step::Version => {
                let list: Vec<_> = match &self.install_play.versions {
                    VersionList::Ready(versions) => versions
                        .all
                        .iter()
                        .filter(|v| v.kind == VersionKind::Release)
                        .take(20)
                        .map(|v| {
                            let id = v.id.clone();
                            shell::row(format!("wizard-v-{}", v.id), false, &t)
                                .flex_none()
                                .h(px(32.))
                                .px(px(12.))
                                .child(
                                    shell::text(theme::UI, shell::c(n.ink)).child(v.id.clone()),
                                )
                                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                    this.install_play.version = Some(id.clone());
                                    this.install_play.java = None;
                                    this.wizard.step = Step::Account;
                                    cx.notify();
                                }))
                                .into_any_element()
                        })
                        .collect(),
                    VersionList::NotAsked | VersionList::Loading => vec![
                        shell::text(theme::UI, shell::c(n.muted))
                            .child(tr("Asking Mojang for its versions…").to_string())
                            .into_any_element(),
                    ],
                    VersionList::Failed(e) => vec![
                        shell::text(theme::UI, shell::c(n.danger))
                            .child(e.clone())
                            .into_any_element(),
                    ],
                };
                body.child(
                    div()
                        .id("wizard-versions")
                        .flex()
                        .flex_col()
                        .h(px(220.))
                        .p(px(4.))
                        .overflow_y_scroll()
                        .rounded(px(theme::night::RADIUS_CONTROL))
                        .border_1()
                        .border_color(plate::alpha(n.stroke, n.stroke_alpha))
                        .children(list),
                )
            }
            Step::Account => body
                .child(
                    shell::text(theme::UI, shell::c(n.ink_soft)).child(
                        tr("Sign in with Ely.by or Microsoft to download game files. You can also play offline with files you already have.")
                            .to_string(),
                    ),
                )
                .child(
                    div().flex().flex_wrap().gap(px(8.)).child(
                        button("wizard-sign-in", "Sign in", Kind::Primary, &t).on_click(
                            cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.sign_in_to_play();
                                cx.notify();
                            }),
                        ),
                    ).child(
                        button("wizard-signed-in", "I'm signed in", Kind::Secondary, &t).on_click(
                            cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.wizard.step = Step::Ready;
                                cx.notify();
                            }),
                        ),
                    ).child(
                        button("wizard-offline", "Continue offline", Kind::Ghost, &t).on_click(
                            cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.settings.dev_offline = true;
                                this.save_settings();
                                this.wizard.step = Step::Ready;
                                cx.notify();
                            }),
                        ),
                    ),
                )
                .child(
                    button("wizard-back", "Back", Kind::Ghost, &t).on_click(cx.listener(
                        |this, _: &ClickEvent, _, cx| {
                            this.wizard.step = Step::Version;
                            cx.notify();
                        },
                    )),
                ),
            Step::Ready => {
                let version = self
                    .install_play
                    .version
                    .clone()
                    .unwrap_or_else(|| tr("latest release").to_string());
                let java = match &self.install_play.java {
                    Some(crate::install_play::JavaState::Ready(need)) => {
                        trf("Java {major} is ready", &[("major", &need.major.to_string())])
                    }
                    Some(crate::install_play::JavaState::Missing(need)) => trf(
                        "Java {major} is missing — Install-and-play on Home offers to install it",
                        &[("major", &need.major.to_string())],
                    ),
                    _ => tr("Java will be checked when the instance is created").to_string(),
                };
                body.child(
                    shell::text(theme::UI, shell::c(n.ink_soft))
                        .child(trf("Minecraft {version}", &[("version", &version)]).to_string()),
                )
                .child(shell::text(theme::CAPTION, shell::c(n.muted)).child(java))
                .child(
                    div().flex().gap(px(8.)).child(
                        button("wizard-create", "Create instance and play", Kind::Primary, &t)
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.wizard.open = false;
                                this.settings.wizard_done = true;
                                this.save_settings();
                                this.install_and_play(cx);
                            })),
                    ).child(
                        button("wizard-back-2", "Back", Kind::Ghost, &t).on_click(cx.listener(
                            |this, _: &ClickEvent, _, cx| {
                                this.wizard.step = Step::Account;
                                cx.notify();
                            },
                        )),
                    ),
                )
            }
        };

        let card = plate(&t)
            .flex()
            .flex_col()
            .gap(px(10.))
            .w(px(520.))
            .max_w_full()
            .p(px(28.))
            .child(body);
        div()
            .id("first-run-wizard")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px(px(32.))
            .py(px(24.))
            .bg(plate::alpha(n.page, 0.92))
            .on_scroll(|_, _, cx| cx.stop_propagation())
            .occlude()
            .child(card)
    }
}
