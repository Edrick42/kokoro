use leptos::prelude::*;
use kokoro_shared::species::Species;
use crate::components::species_card::SpeciesCard;

fn species_css(species: &Species) -> &'static str {
    match species {
        Species::Moluun => "card--moluun",
        Species::Pylum  => "card--pylum",
        Species::Skael  => "card--skael",
        Species::Nyxal  => "card--nyxal",
    }
}

#[component]
pub fn SpeciesListPage() -> impl IntoView {
    view! {
        <h1>"Species"</h1>
        <p>"Every Kobara belongs to one of four species, each shaped by its biome."</p>

        <div class="card-grid">
            {Species::ALL.iter().map(|s| {
                view! {
                    <SpeciesCard
                        name=s.name()
                        classification=s.classification()
                        biome=s.biome()
                        description=s.description()
                        css_class=species_css(s)
                    />
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
