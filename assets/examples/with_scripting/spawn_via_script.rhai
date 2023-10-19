fn update() {
    let spawned_entity = spawn_prototype(world, "Player");
    print(`Spawned entity: ${type_of(spawned_entity)}`);

    world.despawn_recursive(entity);
}