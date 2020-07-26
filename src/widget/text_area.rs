use std::time::Duration;

use druid::{
    Application, BoxConstraints, Cursor, Env, Event, EventCtx, HotKey, KeyCode, LayoutCtx,
    LifeCycle, LifeCycleCtx, PaintCtx, Selector, SysMods, TimerToken, UpdateCtx, Widget,
};

use druid::kurbo::{Affine, Line, Point, RoundedRect, Size, Vec2};
use druid::piet::{
    FontBuilder, PietText, PietTextLayout, RenderContext, Text, TextLayout, TextLayoutBuilder,
};
use druid::text::{
    movement, offset_for_delete_backwards, BasicTextInput, EditAction, EditableText, MouseAction,
    Movement, Selection, TextInput,
};

const BORDER_WIDTH: f64 = 1.;
const PADDING_TOP: f64 = 5.;
const PADDING_LEFT: f64 = 4.;

// we send ourselves this when we want to reset blink, which must be done in event.
const RESET_BLINK: Selector = Selector::new("druid-builtin.reset-textbox-blink");
const CURSOR_BLINK_DRUATION: Duration = Duration::from_millis(500);

/// A multiline text input widget with syntax highlighting for JSON
#[derive(Debug, Clone)]
pub struct TextArea {
    placeholder: String,
    size: Size,
    hscroll_offset: f64,
    selection: Selection,
    cursor_timer: TimerToken,
    cursor_on: bool,
}

impl TextArea {
    /// Perform an `EditAction`. The payload *must* be an `EditAction`.
    pub const PERFORM_EDIT: Selector<EditAction> =
        Selector::new("druid-builtin.textbox.perform-edit");

    /// Create a new TextBox widget
    pub fn new() -> TextArea {
        Self {
            size: Size::ZERO,
            hscroll_offset: 0.,
            selection: Selection::caret(0),
            cursor_timer: TimerToken::INVALID,
            cursor_on: false,
            placeholder: String::new(),
        }
    }

    /// Calculate the PietTextLayout from the given text, font, and font size
    fn get_layout(&self, piet_text: &mut PietText, text: &str, env: &Env) -> PietTextLayout {
        let font_name = env.get(druid::theme::FONT_NAME);
        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);
        // TODO: caching of both the format and the layout
        let font = piet_text
            .new_font_by_name(font_name, font_size)
            .build()
            .unwrap();

        piet_text
            .new_text_layout(&font, &text.to_string(), std::f64::INFINITY)
            .build()
            .unwrap()
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

    /// Return the active edge of the current selection or cursor.
    // TODO: is this the right name?
    fn cursor(&self) -> usize {
        self.selection.end
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
            let cursor = self.cursor();
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
            if text.next_grapheme_offset(self.cursor()).is_some() {
                self.move_selection(Movement::Right, text, false);
                self.delete_backward(text);
            }
        } else {
            self.delete_backward(text);
        }
    }

    /// For a given point, returns the corresponding offset (in bytes) of
    /// the grapheme cluster closest to that point.
    fn offset_for_point(&self, point: Point, layout: &PietTextLayout) -> usize {
        // Translating from screenspace to Piet's text layout representation.
        // We need to account for hscroll_offset state and TextBox's padding.
        let translated_point = Point::new(point.x + self.hscroll_offset - PADDING_LEFT, point.y);
        let hit_test = layout.hit_test_point(translated_point);
        hit_test.metrics.text_position
    }

    /// Given an offset (in bytes) of a valid grapheme cluster, return
    /// the corresponding x coordinate of that grapheme on the screen.
    fn point_for_offset(&self, layout: &PietTextLayout, offset: usize) -> Point {
        if let Some(position) = layout.hit_test_text_position(offset) {
            position.point
        } else {
            //TODO: what is the correct fallback here?
            Point::ZERO
        }
    }

    /// Calculate a stateful scroll offset
    fn update_hscroll(&mut self, layout: &PietTextLayout) {
        let cursor_x = self.point_for_offset(layout, self.cursor()).x;
        let overall_text_width = layout.width();

        let padding = PADDING_LEFT * 2.;
        if overall_text_width < self.size.width {
            // There's no offset if text is smaller than text box
            //
            // [***I*  ]
            // ^
            self.hscroll_offset = 0.;
        } else if cursor_x > self.size.width + self.hscroll_offset - padding {
            // If cursor goes past right side, bump the offset
            //       ->
            // **[****I]****
            //   ^
            self.hscroll_offset = cursor_x - self.size.width + padding;
        } else if cursor_x < self.hscroll_offset {
            // If cursor goes past left side, match the offset
            //    <-
            // **[I****]****
            //   ^
            self.hscroll_offset = cursor_x
        }
    }

    fn reset_cursor_blink(&mut self, ctx: &mut EventCtx) {
        self.cursor_on = true;
        self.cursor_timer = ctx.request_timer(CURSOR_BLINK_DRUATION);
    }
}

impl Widget<String> for TextArea {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut String, env: &Env) {
        // Guard against external changes in data?
        self.selection = self.selection.constrain_to(data);

        let mut text_layout = self.get_layout(&mut ctx.text(), &data, env);
        let mut edit_action = None;

        match event {
            Event::MouseDown(mouse) => {
                ctx.request_focus();
                ctx.set_active(true);

                if !mouse.focus {
                    let cursor_offset = self.offset_for_point(mouse.pos, &text_layout);
                    edit_action = Some(EditAction::Click(MouseAction {
                        row: 0,
                        column: cursor_offset,
                        mods: mouse.mods,
                    }));
                }

                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                ctx.set_cursor(&Cursor::IBeam);
                if ctx.is_active() {
                    let cursor_offset = self.offset_for_point(mouse.pos, &text_layout);
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
                    self.cursor_timer = ctx.request_timer(CURSOR_BLINK_DRUATION);
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
            Event::Command(cmd) if cmd.is(TextArea::PERFORM_EDIT) => {
                let edit = cmd.get_unchecked(TextArea::PERFORM_EDIT);
                self.do_edit_action(edit.to_owned(), data);
            }
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
            _ => (),
        }

        if let Some(edit_action) = edit_action {
            let is_select_all = if let EditAction::SelectAll = &edit_action {
                true
            } else {
                false
            };

            self.do_edit_action(edit_action, data);
            self.reset_cursor_blink(ctx);

            if !is_select_all {
                text_layout = self.get_layout(&mut ctx.text(), &data, env);
                self.update_hscroll(&text_layout);
            }
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
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &String,
        env: &Env,
    ) -> Size {
        let text_layout = self.get_layout(&mut ctx.text(), &data, env);
        let width = env.get(druid::theme::WIDE_WIDGET_WIDTH);
        // TODO use .size.height
        let height = text_layout
            .line_metric(text_layout.line_count() - 1)
            .unwrap()
            .cumulative_height;

        let size = bc.constrain((width, height));
        self.size = size;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &String, env: &Env) {
        // Guard against changes in data following `event`
        let content = if data.is_empty() {
            &self.placeholder
        } else {
            data
        };

        self.selection = self.selection.constrain_to(content);

        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);
        let background_color = env.get(druid::theme::BACKGROUND_LIGHT);
        let selection_color = env.get(druid::theme::SELECTION_COLOR);
        let text_color = env.get(druid::theme::LABEL_COLOR);
        let placeholder_color = env.get(druid::theme::PLACEHOLDER_COLOR);
        let cursor_color = env.get(druid::theme::CURSOR_COLOR);

        let is_focused = ctx.is_focused();

        let border_color = if is_focused {
            env.get(druid::theme::PRIMARY_LIGHT)
        } else {
            env.get(druid::theme::BORDER_DARK)
        };

        // Paint the background
        let clip_rect = RoundedRect::from_origin_size(
            Point::ORIGIN,
            Size::new(
                self.size.width - BORDER_WIDTH,
                self.size.height - BORDER_WIDTH,
            )
            .to_vec2(),
            env.get(druid::theme::TEXTBOX_BORDER_RADIUS),
        );

        ctx.fill(clip_rect, &background_color);

        // Render text, selection, and cursor inside a clip
        ctx.with_save(|rc| {
            rc.clip(clip_rect);

            // Calculate layout
            let text_layout = self.get_layout(&mut rc.text(), &content, env);

            // Shift everything inside the clip by the hscroll_offset
            rc.transform(Affine::translate((-self.hscroll_offset, 0.)));

            // Draw selection rect
            if !self.selection.is_caret() {
                let (left, right) = (self.selection.min(), self.selection.max());
                let left_offset = self.point_for_offset(&text_layout, left);
                let right_offset = self.point_for_offset(&text_layout, right);

                let selection_width = right_offset.x - left_offset.x;
                let selection_height = right_offset.y - left_offset.y;

                let selection_pos = Point::new(
                    left_offset.x + PADDING_LEFT - 1.,
                    left_offset.y + PADDING_TOP - 2.,
                );

                let selection_rect = RoundedRect::from_origin_size(
                    selection_pos,
                    Size::new(selection_width + 2.0, selection_height + 4.).to_vec2(),
                    1.,
                );
                rc.fill(selection_rect, &selection_color);
            }

            // Layout, measure, and draw text
            let text_height = font_size * 0.8;
            let text_pos = Point::new(0.0 + PADDING_LEFT, text_height + PADDING_TOP);
            let color = if data.is_empty() {
                &placeholder_color
            } else {
                &text_color
            };

            rc.draw_text(&text_layout, text_pos, color);

            // Paint the cursor if focused and there's no selection
            if is_focused && self.cursor_on && self.selection.is_caret() {
                let cursor_point = self.point_for_offset(&text_layout, self.cursor());
                let xy = text_pos + Vec2::new(cursor_point.x, cursor_point.y + 2. - font_size);
                let x2y2 = xy + Vec2::new(0., font_size + 2.);
                let line = Line::new(xy, x2y2);

                rc.stroke(line, &cursor_color, 1.);
            }
        });

        // Paint the border
        ctx.stroke(clip_rect, &border_color, BORDER_WIDTH);
    }
}

impl Default for TextArea {
    fn default() -> Self {
        TextArea::new()
    }
}
