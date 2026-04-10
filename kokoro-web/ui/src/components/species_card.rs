use leptos::prelude::*;

#[component]
pub fn SpeciesCard(
    name: &'static str,
    classification: &'static str,
    biome: &'static str,
    description: &'static str,
    css_class: &'static str,
) -> impl IntoView {
    let href = format!("/species/{}", name.to_lowercase());

    view! {
        <a href=href class=format!("card {css_class}") style="display:block;">
            <h3>{name}</h3>
            <p style="font-size:10px;font-family:var(--font-pixel);color:var(--gray);margin-bottom:8px;">
                {classification}
            </p>
            <p>{description}</p>
            <p style="font-size:11px;color:var(--teal);margin-top:8px;">
                {biome}
            </p>
        </a>
    }
}
