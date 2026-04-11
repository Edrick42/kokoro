use leptos::prelude::*;
use crate::server::auth::{fetch_creature, CreatureData};

#[component]
pub fn ProfilePage() -> impl IntoView {
    // Read auth from localStorage on the client
    let (user_name, _set_user_name) = signal(String::new());
    let (user_email, _set_user_email) = signal(String::new());
    let (token, _set_token) = signal(String::new());
    let (logged_in, _set_logged_in) = signal(false);

    // Load from localStorage on mount
    Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                if let Ok(Some(t)) = storage.get_item("kokoro_token") {
                    _set_token.set(t);
                    _set_logged_in.set(true);
                }
                if let Ok(Some(name)) = storage.get_item("kokoro_user") {
                    _set_user_name.set(name);
                }
                if let Ok(Some(email)) = storage.get_item("kokoro_email") {
                    _set_user_email.set(email);
                }
            }
        }
    });

    // Fetch creature data when we have a token
    let creature = Resource::new(
        move || token.get(),
        |t| async move {
            if t.is_empty() {
                return Ok(None);
            }
            fetch_creature(t).await
        },
    );

    let handle_logout = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.remove_item("kokoro_token");
                let _ = storage.remove_item("kokoro_user");
                let _ = storage.remove_item("kokoro_email");
            }
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    };

    view! {
        <div class="profile-page">
            <Show
                when=move || logged_in.get()
                fallback=|| view! {
                    <div class="auth-form">
                        <h2>"Not Logged In"</h2>
                        <p>"You need to log in to see your profile."</p>
                        <a href="/login" class="btn btn--primary" style="display:inline-block;text-align:center;width:100%;margin-top:16px;">"Login"</a>
                    </div>
                }
            >
                <h2>"Profile"</h2>

                <div class="profile-card">
                    <div class="profile-info">
                        <p><strong>"Name: "</strong>{move || user_name.get()}</p>
                        <p><strong>"Email: "</strong>{move || user_email.get()}</p>
                    </div>
                    <button class="btn btn--danger" on:click=handle_logout>"Logout"</button>
                </div>

                <h3>"Your Kobara"</h3>

                <Suspense fallback=|| view! { <p class="loading">"Loading creature data..."</p> }>
                    {move || creature.get().map(|result| match result {
                        Ok(Some(data)) => view! {
                            <div class="creature-card">
                                <div class="creature-info">
                                    <p><strong>"Species: "</strong>{data.species.clone()}</p>
                                    <p><strong>"Age: "</strong>{format!("{} days", data.age_ticks / 86400)}</p>
                                    <p><strong>"Status: "</strong>{if data.alive { "Alive" } else { "Passed away" }}</p>
                                    <p><strong>"Last synced: "</strong>{data.synced_at.clone()}</p>
                                </div>
                                {genome_bars(&data)}
                                {mind_stats(&data)}
                            </div>
                        }.into_any(),
                        Ok(None) => view! {
                            <div class="creature-card empty">
                                <p>"No creature found."</p>
                                <p class="hint">"Play the game to meet your Kobara!"</p>
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <p class="error-msg">{format!("Error: {e}")}</p>
                        }.into_any(),
                    })}
                </Suspense>
            </Show>
        </div>
    }
}

fn genome_bars(data: &CreatureData) -> impl IntoView {
    let genes = [
        ("Curiosity", "curiosity"),
        ("Social Need", "loneliness_sensitivity"),
        ("Appetite", "appetite"),
        ("Circadian", "circadian"),
        ("Resilience", "resilience"),
        ("Learning", "learning_rate"),
    ];

    let bars: Vec<_> = genes.iter().map(|(label, key)| {
        let value = data.genome[key].as_f64().unwrap_or(0.0);
        let pct = (value * 100.0).round() as i64;
        view! {
            <div class="gene-bar">
                <span class="gene-label">{*label}</span>
                <div class="gene-track">
                    <div class="gene-fill" style=format!("width:{}%", pct)></div>
                </div>
            </div>
        }
    }).collect();

    view! { <div class="genome-section"><h4>"Genome"</h4>{bars}</div> }
}

fn mind_stats(data: &CreatureData) -> impl IntoView {
    let hunger = data.mind["hunger"].as_f64().unwrap_or(0.0).round() as i64;
    let happiness = data.mind["happiness"].as_f64().unwrap_or(0.0).round() as i64;
    let energy = data.mind["energy"].as_f64().unwrap_or(0.0).round() as i64;

    view! {
        <div class="mind-stats">
            <h4>"Stats"</h4>
            <p>"Hunger: " <span class="stat-value stat-red">{hunger}</span></p>
            <p>"Happiness: " <span class="stat-value stat-gold">{happiness}</span></p>
            <p>"Energy: " <span class="stat-value stat-teal">{energy}</span></p>
        </div>
    }
}
