use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use kokoro_shared::species::Species;
use kokoro_shared::genes;

#[component]
pub fn SpeciesDetailPage() -> impl IntoView {
    let params = use_params_map();

    let species_view = move || {
        let name = params.read().get("name").unwrap_or_default();
        let species = Species::ALL.iter().find(|s| s.name().to_lowercase() == name.to_lowercase());

        match species {
            Some(s) => {
                let ranges = genes::gene_ranges(s);
                view! {
                    <div class="species-detail">
                        <div class="species-header">
                            <h1>{s.name()}</h1>
                            <p style="font-family:var(--font-pixel);font-size:10px;color:var(--gray);">
                                {s.classification()}
                            </p>
                            <p style="margin-top:12px;">{s.description()}</p>
                            <p style="color:var(--teal);margin-top:8px;">
                                "Biome: " {s.biome()}
                            </p>
                            <p style="font-size:12px;margin-top:8px;">{s.biome_description()}</p>
                        </div>

                        <h2>"Gene Ranges"</h2>
                        <table class="gene-table">
                            <thead>
                                <tr>
                                    <th>"Gene"</th>
                                    <th>"Min"</th>
                                    <th>"Max"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr><td>"Curiosity"</td><td>{format!("{:.1}", ranges.curiosity.0)}</td><td>{format!("{:.1}", ranges.curiosity.1)}</td></tr>
                                <tr><td>"Appetite"</td><td>{format!("{:.1}", ranges.appetite.0)}</td><td>{format!("{:.1}", ranges.appetite.1)}</td></tr>
                                <tr><td>"Resilience"</td><td>{format!("{:.1}", ranges.resilience.0)}</td><td>{format!("{:.1}", ranges.resilience.1)}</td></tr>
                            </tbody>
                        </table>

                        <div style="margin-top:24px;">
                            <a href="/species" class="btn">"Back to Species"</a>
                        </div>
                    </div>
                }.into_any()
            }
            None => view! {
                <h1>"Species not found"</h1>
                <a href="/species" class="btn">"Back to Species"</a>
            }.into_any()
        }
    };

    view! {
        {species_view}
    }
}
