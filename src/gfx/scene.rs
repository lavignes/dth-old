use crate::collections::PoolObject;
use crate::{
    collections::{HashPool, PoolId},
    gfx::{BitmapId, RenderMeshId},
    math::Matrix4,
    math::{Quaternion, Vector3},
};
use smallvec::SmallVec;
use std::cell::{Ref, RefCell, RefMut};

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    position: Vector3,
    scale: Vector3,
    rotation: Quaternion,
}

impl Transform {
    #[inline]
    pub fn concatenated(&self, rhs: &Transform) -> Transform {
        Transform {
            position: self.position + rhs.position,
            scale: self.scale * rhs.scale,
            rotation: self.rotation * rhs.rotation,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Transform::default();
    }

    #[inline]
    pub fn position(&self) -> Vector3 {
        self.position
    }

    #[inline]
    pub fn position_mut(&mut self) -> &mut Vector3 {
        &mut self.position
    }

    #[inline]
    pub fn add_position(&mut self, position: Vector3) {
        self.position += position;
    }

    #[inline]
    pub fn add_rotation(&mut self, rotation: Quaternion) {
        self.rotation *= rotation;
    }
}

impl Into<Matrix4> for Transform {
    #[inline]
    fn into(self) -> Matrix4 {
        &(&Matrix4::scale(self.scale) * &self.rotation.normalized().into())
            * &Matrix4::translate(self.position)
    }
}

impl Default for Transform {
    #[inline]
    fn default() -> Transform {
        Transform {
            position: Vector3::default(),
            scale: Vector3::splat(1.0),
            rotation: Quaternion::identity(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct NodeId(pub u64);

impl PoolId for NodeId {
    #[inline]
    fn next(&self) -> NodeId {
        NodeId(self.0 + 1)
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Todo,
    Mesh(RenderMeshId, BitmapId),
}

impl Default for NodeKind {
    #[inline]
    fn default() -> Self {
        NodeKind::Todo
    }
}

#[derive(Debug, Default)]
pub struct Node {
    parent: Option<NodeId>,
    children: SmallVec<[NodeId; 8]>,
    kind: NodeKind,

    diffuse: Vector3,
    transform: Transform,

    /// This is the transform of the node in world-space. Reading this in the game-logic
    /// would return the world-space matrix from the last render that occurred, so this
    /// value is undefined most of the time.
    world_transform: Transform,
}

impl Node {
    #[inline]
    pub fn new(kind: NodeKind) -> Node {
        Node {
            parent: None,
            children: SmallVec::default(),
            kind,
            diffuse: Vector3::splat(1.0),
            transform: Transform::default(),
            world_transform: Transform::default(),
        }
    }

    #[inline]
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    #[inline]
    pub fn children(&self) -> &[NodeId] {
        &self.children
    }

    /// Get the nth direct child's id
    #[inline]
    pub fn nth_child(&self, n: usize) -> Option<NodeId> {
        self.children.get(n).copied()
    }

    #[inline]
    pub fn set_world_transform(&mut self, transform: &Transform) {
        self.world_transform = *transform
    }

    #[inline]
    pub fn world_transform(&self) -> &Transform {
        &self.world_transform
    }

    #[inline]
    pub fn clear_world_transform(&mut self) {
        self.world_transform.clear();
    }

    #[inline]
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    #[inline]
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl PoolObject for Node {
    #[inline]
    fn clear(&mut self) {
        self.parent = None;
        self.children.clear();
        self.kind = NodeKind::Todo;
        self.diffuse = Vector3::splat(1.0);
        self.transform.clear();
        self.world_transform.clear();
    }
}

#[derive(Default, Debug)]
pub struct SceneGraph {
    nodes: HashPool<NodeId, RefCell<Node>>,
}

impl SceneGraph {
    #[inline]
    pub fn get_node(&self, id: NodeId) -> Option<&RefCell<Node>> {
        self.nodes.get(id)
    }

    #[inline]
    pub fn get_node_ref(&self, id: NodeId) -> Ref<Node> {
        self.nodes.get(id).unwrap().borrow()
    }

    #[inline]
    pub fn get_node_mut_ref(&self, id: NodeId) -> RefMut<Node> {
        self.nodes.get(id).unwrap().borrow_mut()
    }

    #[inline]
    pub fn nodes(&self) -> &HashPool<NodeId, RefCell<Node>> {
        &self.nodes
    }

    /// Sets a node's parent (re-parenting or un-parenting if needed)
    pub fn set_parent(&mut self, node_id: NodeId, parent_id: Option<NodeId>) {
        let old_parent_id: Option<NodeId>;
        {
            let mut node = self.get_node_mut_ref(node_id);
            old_parent_id = node.parent;
            if let Some(parent_id) = parent_id {
                assert_ne!(node_id, parent_id);
                let mut parent_node = self.get_node_mut_ref(parent_id);
                parent_node.children.push(node_id);
            }
            node.parent = parent_id;
        }

        if let Some(old_parent_id) = old_parent_id {
            let mut old_parent_node = self.get_node_mut_ref(old_parent_id);
            let pos = old_parent_node
                .children
                .iter()
                .position(|&child| child == node_id)
                .unwrap();
            old_parent_node.children.remove(pos);
        }
    }
}
