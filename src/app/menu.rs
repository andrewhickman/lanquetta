use druid::MenuDesc;

use crate::app;

pub(in crate::app) fn build() -> MenuDesc<app::State> {
    MenuDesc::empty()
}
