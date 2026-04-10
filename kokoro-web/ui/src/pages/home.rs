use leptos::prelude::*;
use crate::components::species_card::SpeciesCard;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="hero">
            <h1>"Kokoro"</h1>
            <p>"A bio-simulation virtual pet where every creature is genetically unique, biologically realistic, and rendered pixel by pixel in pure Rust."</p>
            <a href="/species" class="btn btn--primary">"Explore Species"</a>
        </div>

        <h2 style="text-align:center;margin-bottom:16px;">"The Four Species"</h2>

        <div class="card-grid">
            <SpeciesCard
                name="Moluun"
                classification="K. moluunaris"
                biome="The Verdance"
                description="Round, soft, forest-dwelling Kobara. Most social manifestation."
                css_class="card--moluun"
            />
            <SpeciesCard
                name="Pylum"
                classification="K. pylumensis"
                biome="Veridian Highlands"
                description="Winged, curious Kobara from the highlands. Reflects the seeking spirit."
                css_class="card--pylum"
            />
            <SpeciesCard
                name="Skael"
                classification="K. skaelith"
                biome="Abyssal Shallows"
                description="Scaled, resilient Kobara from underground caves. Quiet strength."
                css_class="card--skael"
            />
            <SpeciesCard
                name="Nyxal"
                classification="K. nyxalaris"
                biome="Abyssal Depths"
                description="Tentacled, intelligent Kobara from the deep ocean. Mirrors your thinking."
                css_class="card--nyxal"
            />
        </div>
    }
}
