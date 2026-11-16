use gpui_kit::{
    Action, App, Background, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
    assets::IconName,
    component::{ActiveTheme, Icon, Sizable, kbd::Kbd, label::Label},
};

use crate::components::row;

#[derive(IntoElement)]
pub struct LaunchAction {
    id: ElementId,
    icon: IconName,
    label: SharedString,
    action: Option<Box<dyn Action>>,
}

impl LaunchAction {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: IconName::Ban,
            label: "".into(),
            action: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = text.into();
        self
    }

    pub fn action(mut self, action: impl Action) -> Self {
        self.action = Some(Box::new(action));
        self
    }
}

impl RenderOnce for LaunchAction {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let accent: Background = cx.theme().tokens.accent.into();
        let hover_bg = if cx.theme().mode.is_dark() {
            accent.opacity(0.5)
        } else {
            accent
        };

        let shortcut = self
            .action
            .as_ref()
            .and_then(|action| Kbd::global_binding_for_action(action.as_ref(), window));

        row()
            .id(self.id)
            .px_3()
            .py_2()
            .rounded(cx.theme().radius)
            .gap_24()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .child(
                row()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(self.icon).small())
                    .child(Label::new(self.label).text_sm()),
            )
            .children(shortcut)
            .hover(|style| style.bg(hover_bg.clone()))
            .on_click(move |_, window, cx| {
                if let Some(action) = &self.action {
                    window.dispatch_action(action.boxed_clone(), cx)
                }
            })
    }
}
