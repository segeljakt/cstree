mod common;

use common::{
    build_recursive, build_tree_with_cache, Element, SyntaxElement, SyntaxElementRef, SyntaxNode, SyntaxToken,
};
use cstree::{GreenNodeBuilder, NodeCache, SyntaxKind, TextRange};
use lasso::{Resolver, Rodeo};

fn build_tree<D>(root: &Element<'_>) -> (SyntaxNode<D>, impl Resolver) {
    let mut builder = GreenNodeBuilder::new();
    build_recursive(root, &mut builder, 0);
    let (node, interner) = builder.finish();
    (SyntaxNode::new_root(node), interner.unwrap())
}

fn two_level_tree() -> Element<'static> {
    use Element::*;
    Node(vec![
        Node(vec![Token("0.0"), Token("0.1")]),
        Node(vec![Token("1.0")]),
        Node(vec![Token("2.0"), Token("2.1"), Token("2.2")]),
    ])
}

#[test]
fn create() {
    let tree = two_level_tree();
    let (tree, resolver) = build_tree::<()>(&tree);
    assert_eq!(tree.syntax_kind(), SyntaxKind(0));
    assert_eq!(tree.kind(), SyntaxKind(0));
    {
        let leaf1_0 = tree.children().nth(1).unwrap().children_with_tokens().next().unwrap();
        let leaf1_0 = leaf1_0.into_token().unwrap();
        assert_eq!(leaf1_0.syntax_kind(), SyntaxKind(5));
        assert_eq!(leaf1_0.kind(), SyntaxKind(5));
        assert_eq!(leaf1_0.resolve_text(&resolver), "1.0");
        assert_eq!(leaf1_0.text_range(), TextRange::at(6.into(), 3.into()));
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(node2.syntax_kind(), SyntaxKind(6));
        assert_eq!(node2.kind(), SyntaxKind(6));
        assert_eq!(node2.children_with_tokens().count(), 3);
        assert_eq!(node2.resolve_text(&resolver), "2.02.12.2");
    }
}

#[test]
fn data() {
    let tree = two_level_tree();
    let (tree, _resolver) = build_tree::<String>(&tree);
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(*node2.try_set_data("data".into()).unwrap(), "data");
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "data");
        node2.set_data("payload".into());
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "payload");
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert!(node2.try_set_data("already present".into()).is_err());
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "payload");
        node2.set_data("new data".into());
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "new data");
        node2.clear_data();
        // re-use `data` after node data was cleared
        assert_eq!(data.as_str(), "new data");
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(node2.get_data(), None);
    }
}

#[test]
fn with_interner() {
    let mut interner = Rodeo::new();
    let mut cache = NodeCache::with_interner(&mut interner);
    let tree = two_level_tree();
    let tree = build_tree_with_cache(&tree, &mut cache);
    let tree: SyntaxNode = SyntaxNode::new_root(tree);
    let resolver = interner;
    {
        let leaf1_0 = tree.children().nth(1).unwrap().children_with_tokens().next().unwrap();
        let leaf1_0 = leaf1_0.into_token().unwrap();
        assert_eq!(leaf1_0.resolve_text(&resolver), "1.0");
        assert_eq!(leaf1_0.text_range(), TextRange::at(6.into(), 3.into()));
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(node2.resolve_text(&resolver), "2.02.12.2");
    }
}

#[test]
fn inline_resolver() {
    let mut interner = Rodeo::new();
    let mut cache = NodeCache::with_interner(&mut interner);
    let tree = two_level_tree();
    let tree = build_tree_with_cache(&tree, &mut cache);
    let tree: SyntaxNode<(), Rodeo> = SyntaxNode::new_root_with_resolver(tree, interner);
    {
        let leaf1_0 = tree.children().nth(1).unwrap().children_with_tokens().next().unwrap();
        let leaf1_0 = leaf1_0.into_token().unwrap();
        assert_eq!(leaf1_0.text(), "1.0");
        assert_eq!(leaf1_0.text_range(), TextRange::at(6.into(), 3.into()));
        assert_eq!(format!("{}", leaf1_0), leaf1_0.text());
        assert_eq!(format!("{:?}", leaf1_0), "SyntaxKind(5)@6..9 \"1.0\"");
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(node2.text(), "2.02.12.2");
        let resolver = node2.resolver();
        assert_eq!(node2.resolve_text(resolver.as_ref()), node2.text());
        assert_eq!(format!("{}", node2).as_str(), node2.text());
        assert_eq!(format!("{:?}", node2), "SyntaxKind(6)@9..18");
        assert_eq!(
            format!("{:#?}", node2),
            r#"SyntaxKind(6)@9..18
  SyntaxKind(7)@9..12 "2.0"
  SyntaxKind(8)@12..15 "2.1"
  SyntaxKind(9)@15..18 "2.2"
"#
        );
    }
}

#[test]
fn assert_debug_display() {
    use std::fmt;
    fn f<T: fmt::Debug + fmt::Display>() {}

    f::<SyntaxNode<(), lasso::Rodeo>>();
    f::<SyntaxToken<(), lasso::Rodeo>>();
    f::<SyntaxElement<(), lasso::Rodeo>>();
    f::<SyntaxElementRef<'static, (), lasso::Rodeo>>();
    f::<cstree::NodeOrToken<String, u128>>();

    fn dbg<T: fmt::Debug>() {}
    dbg::<GreenNodeBuilder<'static, 'static>>();
}
