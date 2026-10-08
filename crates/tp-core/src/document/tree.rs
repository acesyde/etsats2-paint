//! Operations on a tree of objects (a surface's top-level list and the
//! children of groups).
//!
//! Objects are addressed by id. Mutations walk down with `Arc::make_mut`, so
//! only the edited branch is copied and history snapshots keep sharing the
//! rest; ancestor group frames are refreshed on the way back up.

use std::sync::Arc;

use kurbo::{Point, Rect};

use super::object::{Object, ObjectId, ShapeKind, convex_polygons_overlap};

/// Index path from the top-level list down to an object.
pub type TreePath = Vec<usize>;

/// Result of a hit test: the top-level object a click selects, and the
/// innermost object actually under the point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub top: ObjectId,
    pub inner: ObjectId,
}

/// Where to put moved objects, relative to an existing object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    /// Directly above `ObjectId`, in the same parent.
    Above(ObjectId),
    /// Directly below `ObjectId`, in the same parent.
    Below(ObjectId),
    /// At the top of the group's children.
    IntoTop(ObjectId),
    /// At the top of the top-level list.
    Top,
}

/// Path to the object with `id`, depth-first.
pub fn find_path(list: &[Arc<Object>], id: ObjectId) -> Option<TreePath> {
    for (i, o) in list.iter().enumerate() {
        if o.id == id {
            return Some(vec![i]);
        }
        if let Some(mut rest) = find_path(&o.children, id) {
            rest.insert(0, i);
            return Some(rest);
        }
    }
    None
}

/// The object with `id`, anywhere in the tree.
pub fn get(list: &[Arc<Object>], id: ObjectId) -> Option<&Arc<Object>> {
    let path = find_path(list, id)?;
    let mut node = &list[path[0]];
    for &i in &path[1..] {
        node = &node.children[i];
    }
    Some(node)
}

/// `Some(None)` for a top-level object, `Some(Some(group))` for a child,
/// `None` if the id does not exist.
pub fn parent_of(list: &[Arc<Object>], id: ObjectId) -> Option<Option<ObjectId>> {
    let path = find_path(list, id)?;
    if path.len() == 1 {
        return Some(None);
    }
    let mut node = &list[path[0]];
    for &i in &path[1..path.len() - 1] {
        node = &node.children[i];
    }
    Some(Some(node.id))
}

/// Whether `ancestor` is a strict ancestor of `id`.
pub fn is_ancestor(list: &[Arc<Object>], ancestor: ObjectId, id: ObjectId) -> bool {
    get(list, ancestor).is_some_and(|a| a.id != id && find_path(&a.children, id).is_some())
}

/// Runs `edit` on the child list addressed by `parent` (`[]` = top level),
/// copying only that branch and refreshing ancestor group frames.
pub fn edit_list<R>(
    list: &mut Vec<Arc<Object>>,
    parent: &[usize],
    edit: impl FnOnce(&mut Vec<Arc<Object>>) -> R,
) -> R {
    match parent.split_first() {
        None => edit(list),
        Some((&i, rest)) => {
            let node = Arc::make_mut(&mut list[i]);
            let result = edit_list(&mut node.children, rest, edit);
            node.refresh_group_frame();
            result
        }
    }
}

/// Replaces objects with the same ids, anywhere in the tree. A group is
/// replaced whole, children included.
pub fn replace(list: &mut Vec<Arc<Object>>, objects: &[Object]) {
    for object in objects {
        let Some(path) = find_path(list, object.id) else {
            continue;
        };
        let (last, parent) = path.split_last().expect("non-empty path");
        let mut object = object.clone();
        object.refresh_group_frame();
        edit_list(list, parent, |l| l[*last] = Arc::new(object));
    }
}

/// Removes the given objects (and their subtrees); returns the removed
/// objects in paint order.
pub fn remove(list: &mut Vec<Arc<Object>>, ids: &[ObjectId]) -> Vec<Arc<Object>> {
    let ordered = in_paint_order(list, ids);
    let mut removed = Vec::new();
    for id in ordered {
        if let Some(path) = find_path(list, id) {
            let (last, parent) = path.split_last().expect("non-empty path");
            removed.push(edit_list(list, parent, |l| l.remove(*last)));
        }
    }
    removed
}

/// Paint order (bottom to top, depth-first) of every object in the tree.
pub fn paint_order(list: &[Arc<Object>]) -> Vec<ObjectId> {
    fn walk(list: &[Arc<Object>], out: &mut Vec<ObjectId>) {
        for o in list {
            out.push(o.id);
            walk(&o.children, out);
        }
    }
    let mut out = Vec::new();
    walk(list, &mut out);
    out
}

/// `ids` that exist, sorted bottom to top.
pub fn in_paint_order(list: &[Arc<Object>], ids: &[ObjectId]) -> Vec<ObjectId> {
    paint_order(list)
        .into_iter()
        .filter(|id| ids.contains(id))
        .collect()
}

/// Keeps ids that exist and are not descendants of another selected id.
pub fn normalize_selection(list: &[Arc<Object>], ids: &[ObjectId]) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = Vec::new();
    for &id in ids {
        if out.contains(&id) || get(list, id).is_none() {
            continue;
        }
        if ids
            .iter()
            .any(|&other| other != id && is_ancestor(list, other, id))
        {
            continue;
        }
        out.push(id);
    }
    out
}

/// Inserts objects into `parent` (`None` = top level) at `index` (clamped).
pub fn insert(
    list: &mut Vec<Arc<Object>>,
    parent: Option<ObjectId>,
    index: usize,
    objects: Vec<Arc<Object>>,
) -> bool {
    let path = match parent {
        None => Vec::new(),
        Some(p) => match find_path(list, p) {
            Some(path) if get(list, p).is_some_and(|g| g.is_group()) => path,
            _ => return false,
        },
    };
    edit_list(list, &path, |l| {
        let at = index.min(l.len());
        for (k, o) in objects.into_iter().enumerate() {
            l.insert(at + k, o);
        }
    });
    true
}

/// Moves objects to `placement`, keeping their relative order. Returns false
/// (and changes nothing) when the target is invalid, e.g. a group into itself
/// or its descendants.
pub fn move_to(list: &mut Vec<Arc<Object>>, ids: &[ObjectId], placement: Placement) -> bool {
    let ids = normalize_selection(list, ids);
    if ids.is_empty() {
        return false;
    }
    let anchor = match placement {
        Placement::Above(a) | Placement::Below(a) | Placement::IntoTop(a) => Some(a),
        Placement::Top => None,
    };
    if let Some(anchor) = anchor {
        if get(list, anchor).is_none() {
            return false;
        }
        let anchor_is_moved = ids.contains(&anchor);
        let inside_moved = ids.iter().any(|&id| is_ancestor(list, id, anchor));
        if inside_moved || (anchor_is_moved && matches!(placement, Placement::IntoTop(_))) {
            return false;
        }
        if anchor_is_moved {
            // Dropping next to itself: nothing to do.
            return true;
        }
        if matches!(placement, Placement::IntoTop(_))
            && !get(list, anchor).is_some_and(|g| g.is_group())
        {
            return false;
        }
    }
    let moved = remove(list, &ids);
    match placement {
        Placement::Top => {
            let len = list.len();
            insert(list, None, len, moved)
        }
        Placement::IntoTop(group) => {
            let len = get(list, group).map_or(0, |g| g.children.len());
            insert(list, Some(group), len, moved)
        }
        Placement::Above(a) | Placement::Below(a) => {
            let parent = parent_of(list, a).expect("anchor exists");
            let path = find_path(list, a).expect("anchor exists");
            let index = *path.last().expect("non-empty");
            let at = if matches!(placement, Placement::Above(_)) {
                index + 1
            } else {
                index
            };
            insert(list, parent, at, moved)
        }
    }
}

/// Wraps `ids` in a new group placed where the topmost of them was. Returns
/// the group id, or `None` if nothing was selected.
pub fn group(
    list: &mut Vec<Arc<Object>>,
    ids: &[ObjectId],
    group_id: ObjectId,
) -> Option<ObjectId> {
    let ids = in_paint_order(list, &normalize_selection(list, ids));
    let topmost = *ids.last()?;
    let parent = parent_of(list, topmost).expect("exists");
    // Index of the topmost object once the others are gone.
    let others: Vec<ObjectId> = ids.iter().copied().filter(|id| *id != topmost).collect();
    let removed_others = remove(list, &others);
    let path = find_path(list, topmost).expect("exists");
    let index = *path.last().expect("non-empty");
    let topmost_obj = remove(list, &[topmost]);
    let mut children = removed_others;
    children.extend(topmost_obj);
    let group = Object::group(group_id, children);
    insert(list, parent, index, vec![Arc::new(group)]);
    Some(group_id)
}

/// Replaces each group in `ids` by its children (opacity, visibility and lock
/// are carried into them). Returns the ids of the released children.
pub fn ungroup(list: &mut Vec<Arc<Object>>, ids: &[ObjectId]) -> Vec<ObjectId> {
    let mut released = Vec::new();
    for id in normalize_selection(list, ids) {
        let Some(group) = get(list, id).cloned() else {
            continue;
        };
        if !group.is_group() {
            continue;
        }
        let path = find_path(list, id).expect("exists");
        let (last, parent) = path.split_last().expect("non-empty");
        let children: Vec<Arc<Object>> = group
            .children
            .iter()
            .map(|c| {
                let mut c = (**c).clone();
                c.opacity *= group.opacity;
                c.visible &= group.visible;
                c.locked |= group.locked;
                released.push(c.id);
                Arc::new(c)
            })
            .collect();
        edit_list(list, parent, |l| {
            l.remove(*last);
            for (k, c) in children.into_iter().enumerate() {
                l.insert(*last + k, c);
            }
        });
    }
    released
}

/// Whether the object (or an ancestor) is hidden, and whether it (or an
/// ancestor) is locked.
pub fn effective_flags(list: &[Arc<Object>], id: ObjectId) -> Option<(bool, bool)> {
    let path = find_path(list, id)?;
    let (mut hidden, mut locked) = (false, false);
    let mut nodes = list;
    for &i in &path {
        let node = &nodes[i];
        hidden |= !node.visible;
        locked |= node.locked;
        nodes = &node.children;
    }
    Some((hidden, locked))
}

/// Visible leaves in paint order, with opacity accumulated through groups.
pub fn draw_list(list: &[Arc<Object>]) -> Vec<(Arc<Object>, f32)> {
    fn walk(list: &[Arc<Object>], opacity: f32, out: &mut Vec<(Arc<Object>, f32)>) {
        for o in list.iter().filter(|o| o.visible) {
            let alpha = opacity * o.opacity;
            if o.has_content() {
                walk(&o.children, alpha, out);
            } else {
                out.push((o.clone(), alpha));
            }
        }
    }
    let mut out = Vec::new();
    walk(list, 1.0, &mut out);
    out
}

/// Topmost visible, unlocked leaf under `point`, with its top-level ancestor.
pub fn hit_test(list: &[Arc<Object>], point: Point, tolerance: f64) -> Option<Hit> {
    fn walk(list: &[Arc<Object>], point: Point, tolerance: f64) -> Option<ObjectId> {
        for o in list.iter().rev().filter(|o| o.visible && !o.locked) {
            if o.is_group() {
                if let Some(inner) = walk(&o.children, point, tolerance) {
                    return Some(inner);
                }
            } else if o.contains(point, tolerance) {
                return Some(o.id);
            }
        }
        None
    }
    for o in list.iter().rev().filter(|o| o.visible && !o.locked) {
        let inner = if o.is_group() {
            walk(&o.children, point, tolerance)
        } else {
            o.contains(point, tolerance).then_some(o.id)
        };
        if let Some(inner) = inner {
            return Some(Hit { top: o.id, inner });
        }
    }
    None
}

/// Topmost visible shape under `point`, locked or not (eyedropper sampling).
pub fn sample(list: &[Arc<Object>], point: Point, tolerance: f64) -> Option<&Arc<Object>> {
    for o in list.iter().rev().filter(|o| o.visible) {
        if o.has_content() {
            if let Some(found) = sample(&o.children, point, tolerance) {
                return Some(found);
            }
        } else if o.contains(point, tolerance) {
            return Some(o);
        }
    }
    None
}

/// Visible, unlocked top-level objects with a visible, unlocked part touching
/// `rect`, bottom to top.
pub fn top_level_in_rect(list: &[Arc<Object>], rect: Rect) -> Vec<ObjectId> {
    fn touches(o: &Object, rect: Rect, corners: &[Point]) -> bool {
        if !o.visible || o.locked {
            return false;
        }
        if o.has_content() {
            return o.children.iter().any(|c| touches(c, rect, corners));
        }
        let b = o.bounding_box();
        b.x0 <= rect.x1
            && rect.x0 <= b.x1
            && b.y0 <= rect.y1
            && rect.y0 <= b.y1
            && match o.kind {
                ShapeKind::Polygon { .. } | ShapeKind::Path => o.touches_rect(rect),
                _ => convex_polygons_overlap(&o.flattened(0.5), corners),
            }
    }
    let rect = rect.abs();
    let corners = [
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x1, rect.y0),
        Point::new(rect.x1, rect.y1),
        Point::new(rect.x0, rect.y1),
    ];
    list.iter()
        .filter(|o| touches(o, rect, &corners))
        .map(|o| o.id)
        .collect()
}

/// Moves each selected object one step up within its own parent, past the
/// next unselected sibling.
pub fn bring_forward(list: &mut Vec<Arc<Object>>, ids: &[ObjectId]) {
    reorder(list, ids, true);
}

/// Moves each selected object one step down within its own parent.
pub fn send_backward(list: &mut Vec<Arc<Object>>, ids: &[ObjectId]) {
    reorder(list, ids, false);
}

fn reorder(list: &mut Vec<Arc<Object>>, ids: &[ObjectId], forward: bool) {
    fn step(l: &mut [Arc<Object>], ids: &[ObjectId], forward: bool) {
        let n = l.len();
        if forward {
            for i in (0..n.saturating_sub(1)).rev() {
                if ids.contains(&l[i].id) && !ids.contains(&l[i + 1].id) {
                    l.swap(i, i + 1);
                }
            }
        } else {
            for i in 1..n {
                if ids.contains(&l[i].id) && !ids.contains(&l[i - 1].id) {
                    l.swap(i, i - 1);
                }
            }
        }
    }
    // Collect the distinct parents of the selected objects.
    let mut parents: Vec<TreePath> = Vec::new();
    for &id in ids {
        if let Some(mut path) = find_path(list, id) {
            path.pop();
            if !parents.contains(&path) {
                parents.push(path);
            }
        }
    }
    for parent in parents {
        edit_list(list, &parent, |l| step(l, ids, forward));
    }
}

/// Visible, unlocked top-level object ids (Select All).
pub fn selectable_top_level(list: &[Arc<Object>]) -> Vec<ObjectId> {
    list.iter()
        .filter(|o| o.visible && !o.locked)
        .map(|o| o.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use kurbo::Size;

    use super::*;
    use crate::document::{Frame, ShapeKind};

    fn rect(id: u64, x: f64) -> Arc<Object> {
        Arc::new(Object::new(
            ObjectId(id),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x, 100.0), Size::new(100.0, 100.0), 0.0),
        ))
    }

    /// [1, G10[2, 3], 4] (bottom to top)
    fn sample_tree() -> Vec<Arc<Object>> {
        vec![
            rect(1, 100.0),
            Arc::new(Object::group(
                ObjectId(10),
                vec![rect(2, 300.0), rect(3, 500.0)],
            )),
            rect(4, 700.0),
        ]
    }

    fn ids(list: &[Arc<Object>]) -> Vec<u64> {
        list.iter().map(|o| o.id.0).collect()
    }

    #[test]
    fn find_and_parent() {
        let l = sample_tree();
        assert_eq!(find_path(&l, ObjectId(3)), Some(vec![1, 1]));
        assert_eq!(parent_of(&l, ObjectId(3)), Some(Some(ObjectId(10))));
        assert_eq!(parent_of(&l, ObjectId(4)), Some(None));
        assert!(is_ancestor(&l, ObjectId(10), ObjectId(2)));
        assert!(!is_ancestor(&l, ObjectId(2), ObjectId(10)));
    }

    #[test]
    fn group_frame_follows_children() {
        let l = sample_tree();
        let g = get(&l, ObjectId(10)).unwrap();
        assert_eq!(g.frame.center, Point::new(400.0, 100.0));
        assert_eq!(g.frame.size, Size::new(300.0, 100.0));
    }

    #[test]
    fn edits_refresh_ancestor_frames_and_share_untouched_branches() {
        let mut l = sample_tree();
        let before = l.clone();
        let mut moved = (**get(&l, ObjectId(3)).unwrap()).clone();
        moved.frame.center.x = 900.0;
        replace(&mut l, &[moved]);
        assert_eq!(get(&l, ObjectId(10)).unwrap().frame.size.width, 700.0);
        assert!(Arc::ptr_eq(&before[0], &l[0]), "untouched object is shared");
        assert!(!Arc::ptr_eq(&before[1], &l[1]));
    }

    #[test]
    fn draw_order_descends_into_groups() {
        let l = sample_tree();
        let order: Vec<u64> = draw_list(&l).iter().map(|(o, _)| o.id.0).collect();
        assert_eq!(order, vec![1, 2, 3, 4]);
    }

    #[test]
    fn hidden_and_locked_are_skipped() {
        let mut l = sample_tree();
        let mut hidden = (*l[2]).clone();
        hidden.frame.center = Point::new(300.0, 100.0);
        hidden.visible = false;
        l[2] = Arc::new(hidden);
        let hit = hit_test(&l, Point::new(300.0, 100.0), 0.0).unwrap();
        assert_eq!(
            hit,
            Hit {
                top: ObjectId(10),
                inner: ObjectId(2)
            }
        );
        assert_eq!(draw_list(&l).len(), 3);

        let mut group = (*l[1]).clone();
        group.locked = true;
        l[1] = Arc::new(group);
        assert_eq!(hit_test(&l, Point::new(300.0, 100.0), 0.0), None);
        assert_eq!(effective_flags(&l, ObjectId(2)), Some((false, true)));
    }

    #[test]
    fn sampling_ignores_hidden_but_not_locked() {
        let mut l = sample_tree();
        let mut locked = (*l[2]).clone();
        locked.locked = true;
        l[2] = Arc::new(locked);
        assert_eq!(
            sample(&l, Point::new(700.0, 100.0), 0.0).map(|o| o.id),
            Some(ObjectId(4))
        );
        let mut hidden = (*l[2]).clone();
        hidden.visible = false;
        l[2] = Arc::new(hidden);
        assert_eq!(sample(&l, Point::new(700.0, 100.0), 0.0), None);
    }

    #[test]
    fn hidden_group_hides_children() {
        let mut l = sample_tree();
        let mut g = (*l[1]).clone();
        g.visible = false;
        l[1] = Arc::new(g);
        let order: Vec<u64> = draw_list(&l).iter().map(|(o, _)| o.id.0).collect();
        assert_eq!(order, vec![1, 4]);
        assert_eq!(effective_flags(&l, ObjectId(3)), Some((true, false)));
    }

    #[test]
    fn group_and_ungroup_round_trip() {
        let mut l = sample_tree();
        let g = group(&mut l, &[ObjectId(1), ObjectId(4)], ObjectId(20)).unwrap();
        // Group placed where the topmost (4) was.
        assert_eq!(ids(&l), vec![10, 20]);
        assert_eq!(ids(&get(&l, g).unwrap().children), vec![1, 4]);
        let released = ungroup(&mut l, &[g]);
        assert_eq!(released, vec![ObjectId(1), ObjectId(4)]);
        assert_eq!(ids(&l), vec![10, 1, 4]);
    }

    #[test]
    fn move_into_group_and_reorder() {
        let mut l = sample_tree();
        assert!(move_to(
            &mut l,
            &[ObjectId(4)],
            Placement::IntoTop(ObjectId(10))
        ));
        assert_eq!(ids(&get(&l, ObjectId(10)).unwrap().children), vec![2, 3, 4]);
        assert!(move_to(
            &mut l,
            &[ObjectId(2)],
            Placement::Above(ObjectId(3))
        ));
        assert_eq!(ids(&get(&l, ObjectId(10)).unwrap().children), vec![3, 2, 4]);
        assert!(move_to(
            &mut l,
            &[ObjectId(3)],
            Placement::Below(ObjectId(1))
        ));
        assert_eq!(ids(&l), vec![3, 1, 10]);
    }

    #[test]
    fn group_cannot_move_into_itself_or_descendants() {
        let mut l = sample_tree();
        let before = l.clone();
        assert!(!move_to(
            &mut l,
            &[ObjectId(10)],
            Placement::IntoTop(ObjectId(10))
        ));
        assert!(!move_to(
            &mut l,
            &[ObjectId(10)],
            Placement::Above(ObjectId(2))
        ));
        assert_eq!(l, before);
    }

    #[test]
    fn reorder_within_parent() {
        let mut l = sample_tree();
        bring_forward(&mut l, &[ObjectId(2)]);
        assert_eq!(ids(&get(&l, ObjectId(10)).unwrap().children), vec![3, 2]);
        send_backward(&mut l, &[ObjectId(4)]);
        assert_eq!(ids(&l), vec![1, 4, 10]);
    }

    #[test]
    fn normalize_drops_descendants() {
        let l = sample_tree();
        assert_eq!(
            normalize_selection(&l, &[ObjectId(2), ObjectId(10), ObjectId(99)]),
            vec![ObjectId(10)]
        );
    }

    #[test]
    fn marquee_selects_top_level_groups() {
        let l = sample_tree();
        let hits = top_level_in_rect(&l, Rect::new(450.0, 50.0, 460.0, 60.0));
        assert_eq!(hits, vec![ObjectId(10)]);
    }

    #[test]
    fn remove_returns_paint_order() {
        let mut l = sample_tree();
        let removed = remove(&mut l, &[ObjectId(4), ObjectId(2)]);
        assert_eq!(ids(&removed), vec![2, 4]);
        assert_eq!(ids(&l), vec![1, 10]);
    }
}
