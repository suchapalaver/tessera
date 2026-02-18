//! SDK entry points and builder for composing the block explorer app.

use bevy::prelude::*;

use crate::camera::fly_camera_plugin;
use crate::config;
use crate::data::{init_multi_chain_channel, FetcherConfig};
use crate::render::{BlockRenderer, RendererResource, SlabsAndCubesRenderer};
use crate::scene::{
    arc_plugin, blob_link_plugin, cleanup_old_blocks, heatmap_plugin, ingest_blocks, setup_scene,
};
use crate::ui::{hud_plugin, inspector_plugin, timeline_plugin};

/// Builder for constructing a Tessera app with customizable plugins.
pub struct BlockExplorerBuilder {
    configs: Vec<FetcherConfig>,
    renderer: Option<Box<dyn BlockRenderer>>,
    window_title: String,
    window_resolution: (f32, f32),
    clear_color: Color,
    enable_fly_camera: bool,
    enable_hud: bool,
    enable_inspector: bool,
    enable_timeline: bool,
    enable_arcs: bool,
    enable_heatmap: bool,
    enable_blob_links: bool,
}

impl Default for BlockExplorerBuilder {
    fn default() -> Self {
        Self {
            configs: Vec::new(),
            renderer: None,
            window_title: "Tessera".to_string(),
            window_resolution: (1280.0, 720.0),
            clear_color: Color::srgb(0.05, 0.05, 0.08),
            enable_fly_camera: true,
            enable_hud: true,
            enable_inspector: true,
            enable_timeline: true,
            enable_arcs: true,
            enable_heatmap: true,
            enable_blob_links: true,
        }
    }
}

impl BlockExplorerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use an explicit fetcher configuration (clears previous configs).
    pub fn config(mut self, config: FetcherConfig) -> Self {
        self.configs = vec![config];
        self
    }

    /// Add a single chain configuration.
    pub fn add_chain(mut self, config: FetcherConfig) -> Self {
        self.configs.push(config);
        self
    }

    /// Load all configured chains from environment variables.
    pub fn chain_configs(mut self) -> Self {
        self.configs = config::chain_configs();
        self
    }

    /// Use the default single-chain configuration from environment variables.
    pub fn chain_config(mut self) -> Self {
        self.configs = vec![config::chain_config()];
        self
    }

    pub fn window_title(mut self, title: impl Into<String>) -> Self {
        self.window_title = title.into();
        self
    }

    /// Provide a custom block renderer implementation.
    pub fn renderer(mut self, renderer: impl BlockRenderer) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }

    pub fn window_resolution(mut self, width: f32, height: f32) -> Self {
        self.window_resolution = (width, height);
        self
    }

    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    pub fn disable_fly_camera(mut self) -> Self {
        self.enable_fly_camera = false;
        self
    }

    pub fn disable_hud(mut self) -> Self {
        self.enable_hud = false;
        self
    }

    pub fn disable_inspector(mut self) -> Self {
        self.enable_inspector = false;
        self
    }

    pub fn disable_timeline(mut self) -> Self {
        self.enable_timeline = false;
        self
    }

    pub fn disable_arcs(mut self) -> Self {
        self.enable_arcs = false;
        self
    }

    pub fn disable_heatmap(mut self) -> Self {
        self.enable_heatmap = false;
        self
    }

    pub fn disable_blob_links(mut self) -> Self {
        self.enable_blob_links = false;
        self
    }

    /// Build the Bevy app with the selected configuration and plugins.
    pub fn build(self) -> App {
        let configs = if self.configs.is_empty() {
            config::chain_configs()
        } else {
            self.configs
        };
        let channel = init_multi_chain_channel(configs);
        let renderer = self
            .renderer
            .unwrap_or_else(|| Box::new(SlabsAndCubesRenderer::default()));

        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: self.window_title,
                resolution: self.window_resolution.into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(self.clear_color))
        .insert_resource(channel)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (ingest_blocks, cleanup_old_blocks));

        renderer.setup(&mut app);
        app.insert_resource(RendererResource(renderer));

        if self.enable_fly_camera {
            app.add_plugins(fly_camera_plugin);
        }
        if self.enable_hud {
            app.add_plugins(hud_plugin);
        }
        if self.enable_inspector {
            app.add_plugins(inspector_plugin);
        }
        if self.enable_timeline {
            app.add_plugins(timeline_plugin);
        }
        if self.enable_arcs {
            app.add_plugins(arc_plugin);
        }
        if self.enable_heatmap {
            app.add_plugins(heatmap_plugin);
        }
        if self.enable_blob_links {
            app.add_plugins(blob_link_plugin);
        }

        app
    }
}
