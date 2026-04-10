use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="container">
                <p>"Kokoro — where the spirit lives. Built with Rust."</p>
            </div>
        </footer>
    }
}
