mod actor;
mod geometry;
mod input;
mod mesh;

pub use actor::*;
pub use geometry::*;
pub use input::*;
pub use mesh::*;

use crate::{
    collections::HashPool,
    game::Prefab,
    gfx::Node,
    gfx::{Bitmap, BitmapId, NodeId, RenderMesh, RenderMeshId, SceneGraph},
    math::{Quaternion, Vector3},
};
use std::cell::RefCell;

pub struct Camera {
    position: Vector3,
    rotation: Quaternion,
}

pub struct Engine {
    render_meshes: HashPool<RenderMeshId, RenderMesh>,
    bitmaps: HashPool<BitmapId, Bitmap>,

    node_search_stack: Vec<NodeId>,
    scene: SceneGraph,
    camera: Camera,
    input: Input,

    actors: HashPool<ActorId, RefCell<Actor>>,
    geometry: HashPool<GeometryId, Geometry>,
}

impl Engine {
    /// Traverse the actors, computing their new state per physics update.
    ///
    /// The visitor function allows the renderer to pull out useful shader data at the same time.
    pub fn update_actors<V: FnMut(ActorId, &Actor)>(&mut self, mut renderer_visitor: V) {
        for (id, actor) in self.actors.iter() {
            Prefab::update(*id, &mut actor.borrow_mut(), &self.scene, &self.input);
            renderer_visitor(*id, &actor.borrow());
        }
    }

    /// Traverse the scene graph, computing the new state before a render.
    ///
    /// The visitor function allows the renderer to pull out useful shader data at the same time.
    pub fn update_scene<V: FnMut(NodeId, &Node)>(&mut self, mut visitor: V) {
        // TODO: Can process scene graph in parallel for sure

        // TODO: We could probably keep 2 copies of the graph, one mutable and another immutable
        //   If we kept a copy of the graph that is mutable we could probably reduce
        //   a lot of the borrowing problems because we are trying to mutate while traversing.
        //   Plus we can do interpolation between the graphs.

        // The geoms are a set a of root scene nodes
        for (_, geom) in self.geometry.iter() {
            if let Geometry::StaticMap(map) = geom {
                let id = map.render_node();
                let mut node = self.scene.get_node_mut_ref(id);
                node.clear_world_transform();
                visitor(id, &node);
            } else {
                todo!("{:?}", geom)
            }
        }

        // The actors are a set of root scene nodes
        for (_, actor) in self.actors.iter() {
            let actor = actor.borrow();
            self.node_search_stack.clear();
            let root_world_transform = *actor.transform();

            // We *really* want to know if this fails (see else case)
            if let Some(RenderMode::Node(root_node_id)) = actor.render_mode() {
                self.node_search_stack.push(*root_node_id);

                while let Some(id) = self.node_search_stack.pop() {
                    let mut node = self.scene.get_node_mut_ref(id);
                    // If we don't have a parent then we are a root node
                    let world_transform = if let Some(parent_id) = node.parent() {
                        let parent_node = self.scene.get_node_ref(parent_id);
                        *parent_node.world_transform()
                    } else {
                        root_world_transform
                    };

                    let new_transform = node.transform().concatenated(&world_transform);
                    node.set_world_transform(&new_transform);
                    visitor(id, &node);

                    // Push the children on the stack
                    self.node_search_stack.extend(node.children());
                }
            } else {
                todo!("{:?}", actor.render_mode())
            }
        }
    }
}
