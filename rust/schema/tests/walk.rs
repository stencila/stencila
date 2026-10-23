use stencila_schema::{
    CreativeWorkVariant, ImageObject, Visitor, VisitorMut, WalkControl, WalkNode,
};

#[derive(Default)]
struct Works(usize);

impl Visitor for Works {
    fn visit_work(&mut self, _work: &CreativeWorkVariant) -> WalkControl {
        self.0 += 1;
        WalkControl::Continue
    }
}

impl VisitorMut for Works {
    fn visit_work(&mut self, _work: &mut CreativeWorkVariant) -> WalkControl {
        self.0 += 1;
        WalkControl::Continue
    }
}

#[test]
fn walking_a_creative_work_variant_visits_it() {
    let mut works = vec![
        CreativeWorkVariant::ImageObject(ImageObject::new("a.png".into())),
        CreativeWorkVariant::ImageObject(ImageObject::new("b.png".into())),
    ];

    let mut visitor = Works::default();
    Visitor::walk(&mut visitor, &works);
    assert_eq!(visitor.0, 2);

    let mut visitor = Works::default();
    works.walk_mut(&mut visitor);
    assert_eq!(visitor.0, 2);
}
