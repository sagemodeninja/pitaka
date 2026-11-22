mod pages;

use pages::AccountsPage;

use gpui::{AnyView, AppContext};
use gpui_kit::assets::IconName;
use gpui_kit::component::empty::{
    Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::native_menu::NativeMenu;
use gpui_kit::component::{Icon, TitleBar};
use gpui_kit::{
    App, Context, Div, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement, Render,
    Styled, Window, div,
};

use crate::components::{LaunchAction, column};
use crate::core::state::PitakaState;
use crate::utils::actions::{NewFile, OpenFile};

pub struct MainView {
    state: Entity<PitakaState>,
    pages: Vec<AnyView>,
    active_page: usize,
}

impl MainView {
    pub fn new(state: &Entity<PitakaState>, window: &mut Window, app: &mut App) -> Entity<Self> {
        let pages = vec![AccountsPage::new(state, window, app).into()];

        app.new(|_| Self {
            state: state.clone(),
            pages: pages,
            active_page: 0,
        })
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content = if self.state.read(_cx).vault.is_some() {
            render_main(self)
        } else {
            render_welcome(_cx)
        };

        div()
            .size_full()
            .child(
                TitleBar::new().child("Pitaka").child(
                    div()
                        .on_mouse_down(MouseButton::Left, |event, window, cx| {
                            cx.stop_propagation();

                            NativeMenu::new()
                                .menu("Open Document…", Box::new(OpenFile))
                                .separator()
                                .menu("New Document…", Box::new(NewFile))
                                .show(event.position, window, cx);
                        })
                        .child("Opt"),
                ),
            )
            .child(content)
    }
}

// View when no docs is open.
fn render_welcome(_: &App) -> Div {
    div().size_full().child(
        Empty::new()
            .size_full()
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Icon)
                            .child(Icon::new(IconName::Folder)),
                    )
                    .title(EmptyTitle::new().child("Welcome to Pitaka"))
                    .description(EmptyDescription::new().child(
                        "Open a document or create a new one to start tracking your money.",
                    )),
            )
            .child(
                column().py_5().items_center().justify_center().child(
                    column()
                        .child(
                            LaunchAction::new("launch-action-open")
                                .icon(IconName::FolderOpen)
                                .label("Open Document…")
                                .action(OpenFile),
                        )
                        .child(
                            LaunchAction::new("launch-action-new")
                                .icon(IconName::FilePlusCorner)
                                .label("New Document…")
                                .action(NewFile),
                        ),
                ),
            ),
    )
}

// View when a doc is open.
fn render_main(view: &mut MainView) -> Div {
    div()
        .size_full()
        .child(view.pages.get(view.active_page).cloned().unwrap())
}
