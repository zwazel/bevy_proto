//! This example demonstrates how to create basic schematics using
//! components/resources and the derive macro.

use bevy::{ecs::system::SystemState, prelude::*};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_scripting::{
    api::common::bevy::ScriptWorld,
    prelude::*,
    rhai::{
        rhai::{Dynamic, ImmutableString},
        RhaiContext, RhaiEvent,
    },
};
use bevy_proto::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ProtoPlugin::new(),
            ScriptingPlugin,
            WorldInspectorPlugin::new(),
        ))
        .add_script_host::<RhaiScriptHost<()>>(PostUpdate)
        .add_api_provider::<RhaiScriptHost<()>>(Box::new(RhaiBevyAPIProvider))
        .add_api_provider::<RhaiScriptHost<()>>(Box::new(MyCustomAPI))
        .add_script_handler::<RhaiScriptHost<()>, 0, 0>(PostUpdate)
        // =============== //
        // Make sure to register your types!
        .register_type::<Playable>()
        .register_type::<Alignment>()
        // =============== //
        .add_systems(Startup, (setup, load))
        .add_systems(
            Update,
            (
                call_script_update.run_if(prototype_ready("Player").and_then(run_once())),
                inspect,
            ),
        )
        .run();
}

// A schematic can be pretty much anything that mutates the world.
// The simplest type of a schematic is just a regular Bevy component.
// For components, we can simply add the `Schematic` derive:
#[derive(Component, Schematic)]
// First thing's first, we need to derive `Reflect` so that we can register
// this type to the registry (speaking of, don't forget to do that!):
#[derive(Reflect)]
// Lastly, we need to register `ReflectSchematic`, which can do like this:
#[reflect(Schematic)]
struct Playable;

/// The derive also works for enums!
#[derive(Component, Schematic, Reflect, Debug)]
#[reflect(Schematic)]
enum Alignment {
    Good,
    Neutral,
    Evil,
}

fn load(mut prototypes: PrototypesMut) {
    prototypes.load("examples/with_scripting/PlayerScripting.prototype.ron");
}

fn spawn(world: &mut ScriptWorld, proto_name: ImmutableString) -> Dynamic {
    let mut world = world.write();

    let mut system_state: SystemState<ProtoCommands> = SystemState::new(&mut world);
    let mut proto_commands = system_state.get_mut(&mut world);
    let spawned_mod = proto_commands.spawn(proto_name.as_str()).id();

    Dynamic::from(spawned_mod)
}

// This relies on the `auto_name` feature to be useful
fn inspect(query: Query<(DebugName, &Alignment), Added<Playable>>) {
    for (name, alignment) in &query {
        println!("===============");
        println!("Spawned Player:");
        println!("  ID: {name:?}");
        println!("  Alignment: {alignment:?}");
        println!("===============");
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let script_path = "examples/with_scripting/spawn_via_script.rhai";
    commands.spawn((
        Name::from("ScriptHandler"),
        ScriptCollection::<RhaiFile> {
            scripts: vec![Script::new(
                script_path.to_owned(),
                asset_server.load(script_path),
            )],
        },
    ));
}

fn call_script_update(mut events: PriorityEventWriter<RhaiEvent<()>>) {
    events.send(
        RhaiEvent {
            hook_name: "update".to_owned(),
            args: (),
            recipients: bevy_mod_scripting::prelude::Recipients::All,
        },
        0,
    );
}

#[derive(Default)]
pub struct MyCustomAPI;

impl APIProvider for MyCustomAPI {
    type APITarget = Engine;
    type ScriptContext = RhaiContext;
    type DocTarget = RhaiDocFragment;

    fn attach_api(
        &mut self,
        api: &mut Self::APITarget,
    ) -> Result<(), bevy_mod_scripting::prelude::ScriptError> {
        api.set_max_expr_depths(0, 0);
        api.register_fn("spawn_prototype", spawn);

        Ok(())
    }
}
