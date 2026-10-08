use std::sync::Arc;

use tp_core::document::{Frame, Object, ObjectId, Paint, Rgba, ShapeKind, SymbolId};
use tp_core::kurbo::{Point, Size};
use tp_core::{Project, Surface, TextureResolution};

use crate::workspace::Workspace;

fn ws() -> Workspace {
    let mut p = Project::new("t", TextureResolution::R2048);
    p.surfaces.push(Surface::new("Chassis", 2048.0));
    Workspace::new(p)
}

fn rect(ws: &mut Workspace, x: f64) -> ObjectId {
    ws.create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, 300.0), Size::new(100.0, 50.0), 0.0),
        0.0,
    )
}

fn object(ws: &Workspace, id: ObjectId) -> Object {
    let list = &ws.project.surfaces[ws.project.active_surface].objects;
    (**tp_core::document::tree::get(list, id).unwrap()).clone()
}

/// A symbol made of two rectangles and its instance, selected.
fn logo(ws: &mut Workspace) -> (SymbolId, ObjectId) {
    let a = rect(ws, 300.0);
    let b = rect(ws, 500.0);
    ws.selection = vec![a, b];
    let symbol = ws.convert_to_symbol(1.0).unwrap();
    (symbol, ws.selection[0])
}

#[test]
fn a_change_in_a_symbol_reaches_its_instances_in_one_step() {
    let mut ws = ws();
    let (symbol, instance) = logo(&mut ws);
    ws.edit_symbol(symbol, 2.0);
    assert!(ws.is_editing_symbol());
    let first = ws.project.surface().objects[0].id;
    ws.selection = vec![first];
    ws.apply_color(crate::workspace::ColorTarget::Fill, Rgba::rgb(1, 2, 3));
    ws.commit_pending(3.0);
    ws.finish_symbol_edit(4.0);
    let blue = Paint::Solid(Rgba::rgb(1, 2, 3));
    assert_eq!(object(&ws, instance).children[0].fill, blue);
    ws.undo();
    assert!(ws.is_editing_symbol(), "Undo shows the symbol again");
    let i = &ws.project.surfaces[0].objects[0];
    assert_ne!(i.children[0].fill, blue, "and the instance follows");
}

#[test]
fn moving_an_instance_stays_exact() {
    let mut ws = ws();
    let (symbol, instance) = logo(&mut ws);
    ws.nudge(10.0, 0.0, 2.0);
    let o = object(&ws, instance);
    let ShapeKind::Instance { placement, .. } = o.kind else {
        panic!("an instance");
    };
    let content: Vec<Object> = ws
        .project
        .symbol(symbol)
        .unwrap()
        .surface
        .objects
        .iter()
        .map(|c| (**c).clone())
        .collect();
    let expected = tp_core::document::apply_affine(&content, placement);
    for (child, want) in o.children.iter().zip(&expected) {
        assert_eq!(child.frame, want.frame);
    }
}

#[test]
fn pasting_an_instance_without_its_symbol_gives_a_group() {
    let mut ws = ws();
    let (_, instance) = logo(&mut ws);
    let copied = object(&ws, instance);
    let mut other = Workspace::new(Project::new("other", TextureResolution::R2048));
    other.paste(&[copied], 0.0, 1.0);
    let pasted = other.project.surface().objects[0].clone();
    assert!(pasted.is_group());
    assert_eq!(pasted.children.len(), 2);
}

#[test]
fn undo_across_views_and_choosing_a_texture() {
    let mut ws = ws();
    let (symbol, _) = logo(&mut ws);
    ws.edit_symbol(symbol, 2.0);
    let first = ws.project.surface().objects[0].id;
    ws.selection = vec![first];
    ws.nudge(5.0, 0.0, 3.0);
    ws.finish_symbol_edit(4.0);
    assert!(!ws.is_editing_symbol());
    ws.undo();
    assert_eq!(ws.project.editing_symbol, Some(symbol));
    ws.redo();
    assert_eq!(ws.project.editing_symbol, Some(symbol));
    // Choosing a texture ends the edit.
    ws.set_active_surface(1);
    assert!(!ws.is_editing_symbol());
    assert_eq!(ws.project.active_surface, 1);
    ws.edit_symbol(symbol, 5.0);
    ws.set_active_surface(1);
    assert!(
        !ws.is_editing_symbol(),
        "even the texture it was opened from"
    );
}

#[test]
fn place_detach_duplicate_delete() {
    let mut ws = ws();
    let (symbol, instance) = logo(&mut ws);
    ws.set_active_surface(1);
    ws.place_symbol(symbol, Some(Point::new(1000.0, 1000.0)), 2.0);
    let placed = ws.selection[0];
    assert!(object(&ws, placed).is_instance());
    assert_eq!(ws.project.instance_count(symbol), 2);
    ws.selection = vec![placed];
    ws.detach_selected_instances(3.0);
    assert!(object(&ws, placed).is_group());
    ws.duplicate_symbol(symbol, 4.0);
    assert_eq!(ws.project.symbols.len(), 2);
    ws.set_active_surface(0);
    ws.delete_symbol(symbol, 5.0);
    assert!(object(&ws, instance).is_group());
    ws.undo();
    assert!(object(&ws, instance).is_instance());
    assert!(ws.project.symbol(symbol).is_some());
    let _ = Arc::new(0);
}

fn context(ws: &Workspace) -> crate::commands::EditContext {
    crate::commands::EditContext {
        has_project: true,
        has_selection: !ws.selection.is_empty(),
        editing_symbol: ws.is_editing_symbol(),
        selection_has_instance: ws.selection_has_instance(),
        only_instances: !ws.selection.is_empty()
            && ws.selected_objects().iter().all(|o| o.is_instance()),
        single_instance: ws.selected_instance_symbol().is_some(),
        ..Default::default()
    }
}

#[test]
fn command_reasons_for_symbols_and_instances() {
    use crate::commands::CommandId;
    use crate::state::{disabled_reason_for, is_enabled};
    let mut ws = ws();
    let (symbol, instance) = logo(&mut ws);
    ws.selection = vec![instance];
    let c = context(&ws);
    assert!(is_enabled(CommandId::EditSymbol, &c));
    assert!(is_enabled(CommandId::DetachInstance, &c));
    assert_eq!(
        disabled_reason_for(CommandId::ConvertToPath, &c),
        Some("reason-instance-look")
    );
    ws.edit_symbol(symbol, 2.0);
    let c = context(&ws);
    assert!(!is_enabled(CommandId::CopyFromCabin, &c));
    assert!(!is_enabled(CommandId::ExportTexture, &c));
    assert_eq!(
        disabled_reason_for(CommandId::NextTexture, &c),
        Some("reason-editing-symbol")
    );
    assert!(is_enabled(CommandId::FinishSymbol, &c));
    assert!(
        is_enabled(CommandId::Deselect, &c),
        "Escape leaves the symbol"
    );
}

#[test]
fn colors_apply_to_the_other_objects_only() {
    let mut ws = ws();
    let (_, instance) = logo(&mut ws);
    let r = rect(&mut ws, 1200.0);
    let before = object(&ws, instance);
    ws.selection = vec![r, instance];
    ws.apply_color(crate::workspace::ColorTarget::Fill, Rgba::rgb(9, 9, 9));
    ws.commit_pending(5.0);
    assert_eq!(object(&ws, r).fill, Paint::Solid(Rgba::rgb(9, 9, 9)));
    assert_eq!(object(&ws, instance).children, before.children);
}
