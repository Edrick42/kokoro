use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="nav">
            <div class="container" style="display:flex;align-items:center;justify-content:space-between;">
                <a href="/" class="nav-logo">"Kokoro"</a>
                <ul class="nav-links">
                    <li><a href="/species">"Species"</a></li>
                    <li><a href="/login">"Login"</a></li>
                </ul>
            </div>
        </nav>
    }
}
