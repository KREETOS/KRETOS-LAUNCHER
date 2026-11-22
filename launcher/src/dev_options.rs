//! Local offline play: a Settings section for using an offline name with the Minecraft files
//! already on this computer (`backend::Settings::dev_offline`). Downloading game files still
//! needs a signed-in account (`backend::download_for_owner`).
use super::*;

/// Clicks on the launcher's version that reveal the section.
const CLICKS: u8 = 7;

impl Launcher {
    /// A click on the sidebar's launcher version; the last one opens Settings on the section.
    pub(super) fn version_clicked(&mut self) {
        if self.dev_options {
            return;
        }
        self.dev_clicks += 1;
        if self.dev_clicks >= CLICKS {
            self.dev_options = true;
            self.go(Page::Settings);
        }
    }

    /// The local-offline section in Settings.
    pub(super) fn dev_options_section(&self, cx: &mut Context<Self>) -> Option<Div> {
        let t = self.theme;
        let toggle = switch("dev-offline", self.settings.dev_offline, &t).on_click(cx.listener(
            |this, _: &ClickEvent, _, cx| {
                this.settings.dev_offline = !this.settings.dev_offline;
                this.save_settings();
                cx.notify();
            },
        ));
        Some(
            div()
                .flex()
                .flex_col()
                .child(
                    section_title("Local offline play", "terminal", &t)
                        .mt(px(20.))
                        .mb(px(12.)),
                )
                .child(index_row(
                    0,
                    tr("Play with an offline name"),
                    Some(
                        div()
                            .child(tr(
                                "Uses the Minecraft files already on this computer. \
                                 Downloading game files still needs a signed-in account.",
                            ))
                            .into_any_element(),
                    ),
                    Some(toggle.into_any_element()),
                    &t,
                )),
        )
    }
}
