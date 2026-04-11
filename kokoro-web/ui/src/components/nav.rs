use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    // Check localStorage for auth state (client-side only)
    let (logged_in, _set_logged_in) = signal(false);
    let (user_name, _set_user_name) = signal(String::new());

    Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                if let Ok(Some(_token)) = storage.get_item("kokoro_token") {
                    _set_logged_in.set(true);
                }
                if let Ok(Some(name)) = storage.get_item("kokoro_user") {
                    _set_user_name.set(name);
                }
            }
        }
    });

    view! {
        <nav class="nav">
            <div class="container" style="display:flex;align-items:center;justify-content:space-between;">
                <a href="/" class="nav-logo">"Kokoro"</a>
                <ul class="nav-links">
                    <li><a href="/species">"Species"</a></li>
                    <Show
                        when=move || logged_in.get()
                        fallback=|| view! { <li><a href="/login">"Login"</a></li> }
                    >
                        <li><a href="/profile">{move || user_name.get()}</a></li>
                    </Show>
                </ul>
            </div>
        </nav>
    }
}
