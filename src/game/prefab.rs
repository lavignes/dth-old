use crate::{
    engine::{Actor, ActorId, Input, RenderMode},
    gfx::SceneGraph,
    math::Quaternion,
};

#[derive(Debug)]
pub enum Prefab {}

impl Prefab {
    pub fn clear(&mut self) {}

    pub fn to_actor(&self, _scene: &mut SceneGraph) -> Actor {
        Actor::default()
    }

    pub fn update(_id: ActorId, actor: &mut Actor, scene: &SceneGraph, input: &Input) {
        let lift = if input.button0() {
            0.5
        } else if input.button1() {
            -0.5
        } else {
            0.0
        };
        let stick0 = input.stick0();
        actor
            .transform_mut()
            .add_position((stick0.x(), lift, -stick0.y()).into());

        if let Some(RenderMode::Node(id)) = actor.render_mode() {
            let node = scene.get_node_ref(*id);
            // Example of controlling a child node from a parent
            if let Some(child_id) = node.nth_child(0) {
                let mut node = scene.get_node_mut_ref(child_id);
                node.transform_mut()
                    .add_rotation(Quaternion::from_angle_forward(-0.01));
            }
        }
    }
}
