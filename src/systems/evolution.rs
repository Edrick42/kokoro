fn pet_status_update(mut pet: ResMut<Pet>, time: Res<Time>) {
    // A cada segundo, aumenta a fome
    if time.delta_seconds() > 0.0 {
        pet.hunger = (pet.hunger + 0.1).min(100.0);
    }

    println!("Pet status: {:?}", *pet);
}