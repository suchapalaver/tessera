//! Renderer traits and default implementations.

mod slabs_and_cubes;

use bevy::prelude::*;

use crate::data::BlockPayload;
use crate::scene::blocks::{BlockRegistry, ExplorerState};

pub use slabs_and_cubes::{
    BlobRenderSettings, ClusterLabelSettings, SlabSettings, SlabsAndCubesRenderer,
    SlabsAndCubesSettings, TxRenderSettings,
};

pub trait BlockRenderer: Send + Sync + 'static {
    fn setup(&self, _app: &mut App) {}
    #[allow(clippy::too_many_arguments)]
    fn spawn_block(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        images: &mut ResMut<Assets<Image>>,
        state: &mut ResMut<ExplorerState>,
        registry: &mut ResMut<BlockRegistry>,
        payload: &BlockPayload,
        x_offset: f32,
    );
}

#[derive(Resource)]
pub struct RendererResource(pub Box<dyn BlockRenderer>);

impl RendererResource {
    pub fn new(renderer: impl BlockRenderer) -> Self {
        Self(Box::new(renderer))
    }
}
