use stm32_tui_debugger::tui::widgets::{CompletionItem, CompletionList};

// ─── Helpers ─────────────────────────────────────────────

fn item(text: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        text: text.into(),
        detail: detail.into(),
    }
}

fn sample_items(n: usize) -> Vec<CompletionItem> {
    (0..n)
        .map(|i| item(&format!("item_{i}"), &format!("detail_{i}")))
        .collect()
}

// ─── Construction ────────────────────────────────────────

#[test]
fn new_creates_empty_list() {
    let cl = CompletionList::new();
    assert!(cl.is_empty());
    assert_eq!(cl.len(), 0);
    assert_eq!(cl.selected_index(), 0);
}

#[test]
fn new_has_max_visible_8() {
    let mut cl = CompletionList::new();
    // Populate with 20 items; visible window should be 8
    cl.update(sample_items(20));
    assert_eq!(cl.visible_items().len(), 8);
}

#[test]
fn default_is_same_as_new() {
    let a = CompletionList::new();
    let b = CompletionList::default();
    assert_eq!(a.is_empty(), b.is_empty());
    assert_eq!(a.selected_index(), b.selected_index());
    assert_eq!(a.len(), b.len());
}

#[test]
fn is_empty_true_on_empty() {
    let cl = CompletionList::new();
    assert!(cl.is_empty());
}

#[test]
fn is_active_false_on_empty() {
    let cl = CompletionList::new();
    assert!(!cl.is_active());
}

#[test]
fn is_active_true_with_items() {
    let mut cl = CompletionList::new();
    cl.update(vec![item("a", "")]);
    assert!(cl.is_active());
}

// ─── Update / Clear ─────────────────────────────────────

#[test]
fn update_replaces_items() {
    let mut cl = CompletionList::new();
    cl.update(vec![item("x", "")]);
    assert_eq!(cl.len(), 1);
    assert_eq!(cl.items()[0].text, "x");

    cl.update(vec![item("y", ""), item("z", "")]);
    assert_eq!(cl.len(), 2);
    assert_eq!(cl.items()[0].text, "y");
}

#[test]
fn update_resets_selection_to_zero() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(5));
    cl.select_next();
    cl.select_next();
    assert_eq!(cl.selected_index(), 2);

    cl.update(sample_items(3));
    assert_eq!(cl.selected_index(), 0);
}

#[test]
fn clear_empties_list() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(5));
    cl.clear();
    assert!(cl.is_empty());
    assert_eq!(cl.len(), 0);
    assert_eq!(cl.selected_index(), 0);
}

#[test]
fn update_then_clear() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(3));
    assert!(!cl.is_empty());
    cl.clear();
    assert!(cl.is_empty());
    assert!(!cl.is_active());
}

// ─── Selection Navigation ───────────────────────────────

#[test]
fn select_next_moves_from_0_to_1() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(5));
    assert_eq!(cl.selected_index(), 0);
    cl.select_next();
    assert_eq!(cl.selected_index(), 1);
}

#[test]
fn select_next_wraps_from_last_to_0() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(3));
    cl.select_next(); // 1
    cl.select_next(); // 2
    cl.select_next(); // wraps → 0
    assert_eq!(cl.selected_index(), 0);
}

#[test]
fn select_prev_moves_from_1_to_0() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(5));
    cl.select_next(); // 1
    cl.select_prev(); // 0
    assert_eq!(cl.selected_index(), 0);
}

#[test]
fn select_prev_wraps_from_0_to_last() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(4));
    cl.select_prev(); // wraps → 3
    assert_eq!(cl.selected_index(), 3);
}

#[test]
fn select_next_on_empty_no_panic() {
    let mut cl = CompletionList::new();
    cl.select_next(); // should not panic
    assert_eq!(cl.selected_index(), 0);
}

#[test]
fn select_prev_on_empty_no_panic() {
    let mut cl = CompletionList::new();
    cl.select_prev(); // should not panic
    assert_eq!(cl.selected_index(), 0);
}

#[test]
fn selected_index_tracks_navigation() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(10));
    for i in 0..10 {
        assert_eq!(cl.selected_index(), i);
        cl.select_next();
    }
    // wrapped back to 0
    assert_eq!(cl.selected_index(), 0);
}

// ─── Visible Window & Scrolling ─────────────────────────

#[test]
fn visible_items_all_when_fewer_than_max_visible() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(3));
    assert_eq!(cl.visible_items().len(), 3);
}

#[test]
fn visible_items_capped_at_max_visible() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(15));
    assert_eq!(cl.visible_items().len(), 8); // default max_visible
}

#[test]
fn visible_items_empty_list() {
    let cl = CompletionList::new();
    assert_eq!(cl.visible_items().len(), 0);
}

#[test]
fn scroll_offset_adjusts_on_select_next_past_window() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(12));
    // Navigate to index 8 (past 0..7 window)
    for _ in 0..8 {
        cl.select_next();
    }
    assert_eq!(cl.selected_index(), 8);
    // Visible window should have scrolled to include index 8
    let visible = cl.visible_items();
    assert_eq!(visible.len(), 8);
    assert_eq!(visible.last().unwrap().text, "item_8");
}

#[test]
fn scroll_offset_adjusts_on_select_prev_before_window() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(12));
    // Go to end first (wraps)
    cl.select_prev(); // wraps to 11
    assert_eq!(cl.selected_index(), 11);
    // Navigate backwards several times
    cl.select_prev(); // 10
    cl.select_prev(); // 9
    cl.select_prev(); // 8
    // The visible window should include index 8
    let visible = cl.visible_items();
    assert!(visible.iter().any(|i| i.text == "item_8"));
}

#[test]
fn visible_selected_position_in_window() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(5)); // all visible (5 < 8)
    cl.select_next(); // selected=1
    cl.select_next(); // selected=2
    assert_eq!(cl.visible_selected(), 2);
}

#[test]
fn visible_selected_after_scrolling() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(12));
    // Move to index 9 (should scroll)
    for _ in 0..9 {
        cl.select_next();
    }
    assert_eq!(cl.selected_index(), 9);
    // visible_selected is position within visible window
    let vis_sel = cl.visible_selected();
    assert!(vis_sel < 8);
    // Verify the selected item text in visible window
    let visible = cl.visible_items();
    assert_eq!(visible[vis_sel].text, "item_9");
}

#[test]
fn set_max_visible_changes_window() {
    let mut cl = CompletionList::new();
    cl.set_max_visible(3);
    cl.update(sample_items(10));
    assert_eq!(cl.visible_items().len(), 3);
}

#[test]
fn set_max_visible_minimum_is_1() {
    let mut cl = CompletionList::new();
    cl.set_max_visible(0);
    cl.update(sample_items(5));
    assert_eq!(cl.visible_items().len(), 1);
}

// ─── Accept ─────────────────────────────────────────────

#[test]
fn accept_returns_selected_item() {
    let mut cl = CompletionList::new();
    cl.update(vec![item("alpha", "var"), item("beta", "fn")]);
    let accepted = cl.accept().unwrap();
    assert_eq!(accepted.text, "alpha");
    assert_eq!(accepted.detail, "var");
}

#[test]
fn accept_returns_correct_after_navigation() {
    let mut cl = CompletionList::new();
    cl.update(vec![item("a", ""), item("b", ""), item("c", "")]);
    cl.select_next(); // "b"
    let accepted = cl.accept().unwrap();
    assert_eq!(accepted.text, "b");
}

#[test]
fn accept_on_empty_returns_none() {
    let cl = CompletionList::new();
    assert!(cl.accept().is_none());
}

// ─── Edge Cases ─────────────────────────────────────────

#[test]
fn single_item_next_wraps() {
    let mut cl = CompletionList::new();
    cl.update(vec![item("only", "")]);
    cl.select_next();
    assert_eq!(cl.selected_index(), 0);
    assert_eq!(cl.accept().unwrap().text, "only");
}

#[test]
fn single_item_prev_wraps() {
    let mut cl = CompletionList::new();
    cl.update(vec![item("only", "")]);
    cl.select_prev();
    assert_eq!(cl.selected_index(), 0);
    assert_eq!(cl.accept().unwrap().text, "only");
}

#[test]
fn exactly_max_visible_items_no_scrolling() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(8)); // exactly max_visible
    assert_eq!(cl.visible_items().len(), 8);
    // Navigate to last
    for _ in 0..7 {
        cl.select_next();
    }
    assert_eq!(cl.selected_index(), 7);
    assert_eq!(cl.visible_items().len(), 8);
    assert_eq!(cl.visible_selected(), 7);
}

#[test]
fn max_visible_plus_one_scrolls() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(9)); // one more than max_visible
    // Navigate to index 8
    for _ in 0..8 {
        cl.select_next();
    }
    assert_eq!(cl.selected_index(), 8);
    // Window should have shifted: items 1..=8
    let visible = cl.visible_items();
    assert_eq!(visible.len(), 8);
    assert_eq!(visible[7].text, "item_8");
}

#[test]
fn wrap_forward_resets_scroll_offset() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(12));
    // Navigate to end (scrolls)
    for _ in 0..11 {
        cl.select_next();
    }
    assert_eq!(cl.selected_index(), 11);
    // Wrap forward to 0
    cl.select_next();
    assert_eq!(cl.selected_index(), 0);
    // First item should be visible
    assert_eq!(cl.visible_items()[0].text, "item_0");
    assert_eq!(cl.visible_selected(), 0);
}

#[test]
fn wrap_backward_adjusts_scroll_to_end() {
    let mut cl = CompletionList::new();
    cl.update(sample_items(12));
    // From 0, go prev → wraps to 11
    cl.select_prev();
    assert_eq!(cl.selected_index(), 11);
    // Last item should be visible
    let visible = cl.visible_items();
    assert_eq!(visible.last().unwrap().text, "item_11");
}

#[test]
fn items_accessor_returns_all() {
    let mut cl = CompletionList::new();
    let items = vec![item("a", "1"), item("b", "2"), item("c", "3")];
    cl.update(items);
    assert_eq!(cl.items().len(), 3);
    assert_eq!(cl.items()[1].text, "b");
}

#[test]
fn len_matches_items_count() {
    let mut cl = CompletionList::new();
    assert_eq!(cl.len(), 0);
    cl.update(sample_items(7));
    assert_eq!(cl.len(), 7);
    cl.clear();
    assert_eq!(cl.len(), 0);
}
