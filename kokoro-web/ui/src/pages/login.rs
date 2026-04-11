use leptos::prelude::*;
use crate::server::auth::Login;

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<Login>::new();
    let value = login_action.value();

    // On successful login, store session in localStorage and redirect
    Effect::new(move || {
        if let Some(Ok(_session)) = value.get() {
            // Store token in localStorage for client-side access
            #[cfg(feature = "hydrate")]
            {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                {
                    let _ = storage.set_item("kokoro_token", &_session.token);
                    let _ = storage.set_item("kokoro_user", &_session.display_name);
                    let _ = storage.set_item("kokoro_email", &_session.email);
                }
                // Redirect to profile
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/profile");
                }
            }
        }
    });

    let error_msg = move || {
        value.get().and_then(|r: Result<_, _>| r.err()).map(|e| e.to_string())
    };

    view! {
        <div class="auth-form">
            <h2>"Login"</h2>

            {move || error_msg().map(|msg| view! {
                <p class="error-msg">{msg}</p>
            })}

            <ActionForm action=login_action>
                <div class="form-group">
                    <label>"Email"</label>
                    <input type="email" name="email" placeholder="your@email.com" required/>
                </div>

                <div class="form-group">
                    <label>"Password"</label>
                    <input type="password" name="password" placeholder="********" required/>
                </div>

                <button type="submit" class="btn btn--primary" style="width:100%;">
                    {move || if login_action.pending().get() { "Logging in..." } else { "Login" }}
                </button>
            </ActionForm>

            <p style="text-align:center;margin-top:16px;font-size:12px;">
                "No account? " <a href="/register">"Register"</a>
            </p>
        </div>
    }
}
