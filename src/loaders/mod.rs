// TODO: Most of these should go away once we get BSN

mod faceshapes;
mod mhclo;
mod mhmat;
mod morph_target;
mod obj_base_mesh;
mod proxy;
mod pose;
mod thumb;
mod vertex_groups;
mod skin_weights;
mod rig;

#[allow(unused_imports)]
pub use self::{
    faceshapes::*,
    mhclo::*,
    mhmat::*,
    morph_target::*,
    obj_base_mesh::*,
    proxy::*,
    pose::*,
    thumb::*,
    vertex_groups::*,
    skin_weights::*,
    rig::*,
};
