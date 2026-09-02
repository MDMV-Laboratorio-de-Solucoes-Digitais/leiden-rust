//! Integration tests: `Point2D` geometry operations and `TerminalDimensionGuard` (T005).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::Point2D;
use leiden_tui::TerminalDimensionGuard;
use ratatui::layout::Rect;

#[test]
fn point2d_distance_to() {
    let a = Point2D::new(0.0, 0.0);
    let b = Point2D::new(3.0, 4.0);
    assert!((a.distance_to(b) - 5.0).abs() < 1e-9);

    let c = Point2D::new(1.0, 1.0);
    assert!((c.distance_to(c) - 0.0).abs() < 1e-9);
}

#[test]
fn point2d_distance_to_symmetric() {
    let a = Point2D::new(0.3, 0.5);
    let b = Point2D::new(0.7, 0.2);
    let d1 = a.distance_to(b);
    let d2 = b.distance_to(a);
    assert!((d1 - d2).abs() < 1e-9);
}

#[test]
fn point2d_add_scaled() {
    let base = Point2D::new(1.0, 2.0);
    let vec = Point2D::new(3.0, 4.0);
    let result = base.add_scaled(vec, 2.0);
    assert!((result.x - 7.0).abs() < 1e-9);
    assert!((result.y - 10.0).abs() < 1e-9);
}

#[test]
fn point2d_add_scaled_zero() {
    let base = Point2D::new(5.0, 5.0);
    let vec = Point2D::new(3.0, 4.0);
    let result = base.add_scaled(vec, 0.0);
    assert!((result.x - 5.0).abs() < 1e-9);
    assert!((result.y - 5.0).abs() < 1e-9);
}

#[test]
fn point2d_clamp_within_bounds() {
    let p = Point2D::new(0.5, 0.5);
    let clamped = p.clamp(0.0, 1.0, 0.0, 1.0);
    assert!((clamped.x - 0.5).abs() < 1e-9);
    assert!((clamped.y - 0.5).abs() < 1e-9);
}

#[test]
fn point2d_clamp_below_bounds() {
    let p = Point2D::new(-1.0, -2.0);
    let clamped = p.clamp(0.0, 1.0, 0.0, 1.0);
    assert!((clamped.x - 0.0).abs() < 1e-9);
    assert!((clamped.y - 0.0).abs() < 1e-9);
}

#[test]
fn point2d_clamp_above_bounds() {
    let p = Point2D::new(5.0, 10.0);
    let clamped = p.clamp(0.0, 1.0, 0.0, 1.0);
    assert!((clamped.x - 1.0).abs() < 1e-9);
    assert!((clamped.y - 1.0).abs() < 1e-9);
}

#[test]
fn terminal_dimension_guard_standard_80x24() {
    let guard = TerminalDimensionGuard::standard();
    assert_eq!(guard.min_columns, 80);
    assert_eq!(guard.min_rows, 24);
}

#[test]
fn terminal_dimension_guard_is_valid_sufficient() {
    let guard = TerminalDimensionGuard::standard();
    assert!(guard.is_valid(80, 24));
    assert!(guard.is_valid(120, 40));
    assert!(guard.is_valid(200, 100));
}

#[test]
fn terminal_dimension_guard_is_valid_too_small() {
    let guard = TerminalDimensionGuard::standard();
    assert!(!guard.is_valid(79, 24));
    assert!(!guard.is_valid(80, 23));
    assert!(!guard.is_valid(72, 20));
    assert!(!guard.is_valid(0, 0));
}

#[test]
fn terminal_dimension_guard_is_area_valid() {
    let guard = TerminalDimensionGuard::standard();
    let good_area = Rect::new(0, 0, 80, 24);
    let small_area = Rect::new(0, 0, 72, 20);

    assert!(guard.is_area_valid(good_area));
    assert!(!guard.is_area_valid(small_area));
}

#[test]
fn terminal_dimension_guard_is_area_valid_large() {
    let guard = TerminalDimensionGuard::standard();
    let large_area = Rect::new(0, 0, 200, 60);
    assert!(guard.is_area_valid(large_area));
}

#[test]
fn terminal_dimension_guard_default_is_standard() {
    let guard = TerminalDimensionGuard::default();
    assert_eq!(guard.min_columns, 80);
    assert_eq!(guard.min_rows, 24);
    assert!(guard.is_valid(80, 24));
}

#[test]
fn terminal_dimension_guard_edge_boundary() {
    let guard = TerminalDimensionGuard::standard();
    // Exactly at minimum — should be valid
    assert!(guard.is_valid(80, 24));
    // One less in either dimension — invalid
    assert!(!guard.is_valid(79, 24));
    assert!(!guard.is_valid(80, 23));
}
