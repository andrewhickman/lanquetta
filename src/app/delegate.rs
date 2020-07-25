use druid::AppDelegate;

use crate::app;

pub(in crate::app) fn build() -> impl AppDelegate<app::State> {
    Delegate
}

struct Delegate;

impl AppDelegate<app::State> for Delegate {}
