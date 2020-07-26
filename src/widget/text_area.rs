use std::time::Duration;

use druid::piet::{
    FontBuilder, PietText, PietTextLayout, RenderContext, Text, TextLayout, TextLayoutBuilder,
};
use druid::text::{
    movement, offset_for_delete_backwards, BasicTextInput, EditAction, EditableText, MouseAction,
    Movement, Selection, TextInput,
};
use druid::{
    kurbo::Line,
    widget::{Container, Scroll, TextBox},
    Application, BoxConstraints, Cursor, Env, Event, EventCtx, HotKey, KeyCode, LayoutCtx,
    LifeCycle, LifeCycleCtx, PaintCtx, Point, Selector, Size, SysMods, TimerToken, UpdateCtx,
    Widget, WidgetExt,
};

// we send ourselves this when we want to reset blink, which must be done in event.
const RESET_BLINK: Selector = Selector::new("druid-builtin.reset-textbox-blink");
const CURSOR_BLINK_DURATION: Duration = Duration::from_millis(500);

/// A multiline text input widget with syntax highlighting for JSON
#[derive(Clone)]
pub struct TextArea {
    size: Size,
    selection: Selection,
    cursor_on: bool,
    cursor_timer: TimerToken,
}

impl TextArea {
    pub fn new() -> Self {
        TextArea {
            size: Size::ZERO,
            selection: Selection::caret(0),
            cursor_on: false,
            cursor_timer: TimerToken::INVALID,
        }
    }

    pub fn styled(self) -> impl Widget<String> {
        Container::new(
            Scroll::new(self.padding((4.0, 3.0)))
                .vertical()
                .expand_height(),
        )
        .rounded(2.0)
        .background(druid::theme::BACKGROUND_LIGHT)
        .border(druid::theme::BORDER_DARK, 1.0)
    }

    fn get_layout(
        &self,
        piet_text: &mut PietText,
        text: &str,
        env: &Env,
        width: f64,
    ) -> PietTextLayout {
        let font_name = env.get(druid::theme::FONT_NAME);
        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);
        // TODO: caching of both the format and the layout
        let font = piet_text
            .new_font_by_name(font_name, font_size)
            .build()
            .unwrap();

        piet_text
            .new_text_layout(&font, &text.to_string(), width)
            .build()
            .unwrap()
    }

    fn do_edit_action(&mut self, edit_action: EditAction, text: &mut String) {
        match edit_action {
            EditAction::Insert(chars) | EditAction::Paste(chars) => self.insert(text, &chars),
            EditAction::Backspace => self.delete_backward(text),
            EditAction::Delete => self.delete_forward(text),
            EditAction::Move(movement) => self.move_selection(movement, text, false),
            EditAction::ModifySelection(movement) => self.move_selection(movement, text, true),
            EditAction::SelectAll => self.selection.all(text),
            EditAction::Click(action) => {
                if action.mods.shift {
                    self.selection.end = action.column;
                } else {
                    self.caret_to(text, action.column);
                }
            }
            EditAction::Drag(action) => self.selection.end = action.column,
        }
    }

    /// Insert text at the cursor position.
    /// Replaces selected text if there's a selection.
    fn insert(&mut self, src: &mut String, new: &str) {
        // EditableText's edit method will panic if selection is greater than
        // src length, hence we try to constrain it.
        //
        // This is especially needed when data was modified externally.
        // TODO: perhaps this belongs in update?
        let selection = self.selection.constrain_to(src);

        src.edit(selection.range(), new);
        self.selection = Selection::caret(selection.min() + new.len());
    }

    /// Set the selection to be a caret at the given offset, if that's a valid
    /// codepoint boundary.
    fn caret_to(&mut self, text: &mut String, to: usize) {
        match text.cursor(to) {
            Some(_) => self.selection = Selection::caret(to),
            None => log::error!("You can't move the cursor there."),
        }
    }

    /// Edit a selection using a `Movement`.
    fn move_selection(&mut self, mvmnt: Movement, text: &mut String, modify: bool) {
        // This movement function should ensure all movements are legit.
        // If they aren't, that's a problem with the movement function.
        self.selection = movement(mvmnt, self.selection, text, modify);
    }

    /// Delete to previous grapheme if in caret mode.
    /// Otherwise just delete everything inside the selection.
    fn delete_backward(&mut self, text: &mut String) {
        if self.selection.is_caret() {
            let cursor = self.selection.end;
            let new_cursor = offset_for_delete_backwards(&self.selection, text);
            text.edit(new_cursor..cursor, "");
            self.caret_to(text, new_cursor);
        } else {
            text.edit(self.selection.range(), "");
            self.caret_to(text, self.selection.min());
        }
    }

    fn delete_forward(&mut self, text: &mut String) {
        if self.selection.is_caret() {
            // Never touch the characters before the cursor.
            if text.next_grapheme_offset(self.selection.end).is_some() {
                self.move_selection(Movement::Right, text, false);
                self.delete_backward(text);
            }
        } else {
            self.delete_backward(text);
        }
    }

    fn reset_cursor_blink(&mut self, ctx: &mut EventCtx) {
        self.cursor_on = true;
        self.cursor_timer = ctx.request_timer(CURSOR_BLINK_DURATION);
    }
}

impl Widget<String> for TextArea {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut String, env: &Env) {
        self.selection = self.selection.constrain_to(data);

        let text_layout = self.get_layout(&mut ctx.text(), &data, env, self.size.width);
        let mut edit_action = None;

        match event {
            Event::MouseDown(mouse) => {
                ctx.request_focus();
                ctx.set_active(true);

                if !mouse.focus {
                    let cursor_offset = text_layout.hit_test_point(mouse.pos).metrics.text_position;
                    edit_action = Some(EditAction::Click(MouseAction {
                        row: 2,
                        column: cursor_offset,
                        mods: mouse.mods,
                    }));
                }

                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                ctx.set_cursor(&Cursor::IBeam);
                if ctx.is_active() {
                    let cursor_offset = text_layout.hit_test_point(mouse.pos).metrics.text_position;
                    edit_action = Some(EditAction::Drag(MouseAction {
                        row: 0,
                        column: cursor_offset,
                        mods: mouse.mods,
                    }));
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    ctx.request_paint();
                }
            }
            Event::Timer(id) => {
                if *id == self.cursor_timer {
                    self.cursor_on = !self.cursor_on;
                    ctx.request_paint();
                    self.cursor_timer = ctx.request_timer(CURSOR_BLINK_DURATION);
                }
            }
            Event::Command(ref cmd)
                if ctx.is_focused()
                    && (cmd.is(druid::commands::COPY) || cmd.is(druid::commands::CUT)) =>
            {
                if let Some(text) = data.slice(self.selection.range()) {
                    Application::global().clipboard().put_string(text);
                }
                if !self.selection.is_caret() && cmd.is(druid::commands::CUT) {
                    edit_action = Some(EditAction::Delete);
                }
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(RESET_BLINK) => self.reset_cursor_blink(ctx),
            Event::Command(cmd) if cmd.is(TextBox::PERFORM_EDIT) => {
                let edit = cmd.get_unchecked(TextBox::PERFORM_EDIT);
                self.do_edit_action(edit.to_owned(), data);
            }
            Event::Command(_) => {}
            Event::Paste(ref item) => {
                if let Some(string) = item.get_string() {
                    edit_action = Some(EditAction::Paste(string));
                    ctx.request_paint();
                }
            }
            Event::KeyDown(key_event) => {
                let event_handled = match key_event {
                    // Tab and shift+tab
                    k_e if HotKey::new(None, KeyCode::Tab).matches(k_e) => {
                        ctx.focus_next();
                        true
                    }
                    k_e if HotKey::new(SysMods::Shift, KeyCode::Tab).matches(k_e) => {
                        ctx.focus_prev();
                        true
                    }
                    _ => false,
                };

                if !event_handled {
                    edit_action = BasicTextInput::new().handle_event(key_event);
                }

                ctx.request_paint();
            }

            Event::WindowConnected => {}
            Event::WindowSize(_) => {}
            Event::Wheel(_) => {}
            Event::KeyUp(_) => {}
            Event::Zoom(_) => {}
            Event::Internal(_) => {}
        }

        if let Some(edit_action) = edit_action {
            self.do_edit_action(edit_action, data);
            self.reset_cursor_blink(ctx);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &String, _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => ctx.register_for_focus(),
            // an open question: should we be able to schedule timers here?
            LifeCycle::FocusChanged(true) => ctx.submit_command(RESET_BLINK, ctx.widget_id()),
            _ => (),
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &String, _data: &String, _env: &Env) {
        ctx.request_paint();
        ctx.request_layout();
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &String,
        env: &Env,
    ) -> Size {
        let width = bc.max().width;
        let text_layout = self.get_layout(&mut ctx.text(), data, env, width);
        let height = if let Some(last_line_idx) = text_layout.line_count().checked_sub(1) {
            text_layout
                .line_metric(last_line_idx)
                .unwrap()
                .cumulative_height
        } else {
            0.0
        };

        self.size = bc.constrain((width, height));
        assert_eq!(self.size.width, width);
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &String, env: &Env) {
        ctx.with_save(|rc| {
            rc.clip(self.size.to_rect());

            let text_layout = self.get_layout(&mut rc.text(), data, env, self.size.width);
            let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);

            rc.draw_text(
                &text_layout,
                Point::new(0.0, font_size),
                &env.get(druid::theme::LABEL_COLOR),
            );

            if self.cursor_on && self.selection.is_caret() {
                if let Some(position) = text_layout.hit_test_text_position(self.selection.start) {
                    let start = Point::new(position.point.x, position.point.y);
                    let end = Point::new(position.point.x, position.point.y + font_size);
                    let line = Line::new(start, end);

                    rc.stroke(line, &env.get(druid::theme::CURSOR_COLOR), 1.);
                }
            }
        });
    }
}

impl Default for TextArea {
    fn default() -> Self {
        TextArea::new()
    }
}
