use gpui_kit::component::Icon;

macro_rules! custom_icons {
    ($($name:ident => $file:literal),+ $(,)?) => {
        pub enum CustomIcon {
            $($name),+
        }

        impl CustomIcon {
            fn bytes(self) -> &'static [u8] {
                match self {
                    $(Self::$name => include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/icons/",
                        $file
                    ))),+
                }
            }
        }
    };
}

custom_icons! {
    Trash => "trash.svg",
}

impl Into<Icon> for CustomIcon {
    fn into(self) -> Icon {
        Icon::default().data(self.bytes())
    }
}
